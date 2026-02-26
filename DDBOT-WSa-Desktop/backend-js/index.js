const express = require('express');
const cors = require('cors');
const fs = require('fs-extra');
const path = require('path');
const { spawn } = require('child_process');
const dayjs = require('dayjs');
const betterSqlite3 = require('better-sqlite3');
const https = require('https');
const AdmZip = require('adm-zip');
const tar = require('tar');
const axios = require('axios');
const { createProxyMiddleware } = require('http-proxy-middleware');
const cookieParser = require('cookie-parser');

const app = express();
const PORT = process.env.PORT || 3000;

// Default Admin Password (Can be overridden by env variable or local config file)
let ADMIN_PASSWORD = process.env.WEBUI_PASSWORD || 'ddbot_admin';
try {
    const cfgPath = path.join(__dirname, 'server-config.json');
    if (fs.existsSync(cfgPath)) {
        const localConfig = JSON.parse(fs.readFileSync(cfgPath, 'utf8'));
        if (localConfig.ADMIN_PASSWORD) {
            ADMIN_PASSWORD = localConfig.ADMIN_PASSWORD;
        }
    }
} catch (e) {
    // Ignore initial parse errors
}

// Check for duplicate middlewares and remove them

// Middleware to parse cookies
app.use(cookieParser());
const authMiddleware = (req, res, next) => {
    // Determine the true client IP (handling reverse proxies if configured)
    const clientIp = req.headers['x-forwarded-for'] || req.socket.remoteAddress || req.ip;

    // Normalize IPv6 mapped IPv4
    const normalizedIp = clientIp.replace(/^.*:/, '');

    const isPrivateIp = (ip) => {
        return ip === '127.0.0.1' ||
            ip === 'localhost' ||
            ip.startsWith('192.168.') ||
            ip.startsWith('10.') ||
            /^172\.(1[6-9]|2[0-9]|3[0-1])\./.test(ip);
    };

    if (isPrivateIp(normalizedIp)) {
        // Internal IP -> Bypass Auth
        return next();
    }

    // Always allow the login route so users can authenticate
    if (req.path === '/api/auth/login') {
        return next();
    }

    // External IP -> Require Auth Token
    const token = req.cookies.auth_token || req.headers['authorization'];
    if (token === ADMIN_PASSWORD) {
        return next();
    }

    // Unauthenticated
    // Distinguish between API requests and page requests
    if (req.path.startsWith('/api')) {
        return res.status(401).json({ error: 'Unauthorized: External access requires authentication.' });
    } else {
        // For HTML page requests, let it serve the index, the Vue router will handle the login redirect if API fails
        return next();
    }
};

app.use(authMiddleware);

// Set up Reverse Proxy for the Go Admin API BEFORE body parsers to stream smoothly
app.use('/api/v1', createProxyMiddleware({
    target: 'http://127.0.0.1:15631',
    changeOrigin: true,
    pathRewrite: { '^/api/v1': '/api/v1' } // Preserve path
}));

app.use('/api/admin', createProxyMiddleware({
    target: 'http://127.0.0.1:15631',
    changeOrigin: true,
    pathRewrite: { '^/api/admin': '/api/admin' } // Preserve path
}));

// 静态文件托管：将前端构建产物（dist）直接部署在根路径
const distPath = path.join(__dirname, '..', 'dist');
app.use(express.static(distPath));

app.use(cors());
app.use(express.json());

// 全局状态管理
let processState = null;

// 路径约定
const getWorkdir = () => path.resolve(__dirname, '..');
const getDdbotDataDir = () => path.resolve(getWorkdir(), 'data', 'ddbot');
const getBackupsDir = () => path.resolve(getDdbotDataDir(), 'backups');

// 辅助函数：查找可执行文件
function findDdbotExecutable(dir) {
    if (!fs.existsSync(dir)) return null;
    const files = fs.readdirSync(dir);
    for (const file of files) {
        const lowerName = file.toLowerCase();
        if (process.platform === 'win32') {
            if (lowerName.endsWith('ddbot.exe') || lowerName.endsWith('ddbot-wsa.exe')) {
                return path.join(dir, file);
            }
        } else {
            if (lowerName === 'ddbot' || lowerName === 'ddbot-wsa') {
                return path.join(dir, file);
            }
        }
    }
    return null;
}

// APIs

// Login Route for External Users
app.post('/api/auth/login', (req, res) => {
    const { password } = req.body;
    if (password === ADMIN_PASSWORD) {
        // Issue token as an HttpOnly cookie valid for 30 days
        res.cookie('auth_token', password, {
            maxAge: 30 * 24 * 60 * 60 * 1000,
            httpOnly: false, // Accessible by frontend if needed, or keeping it strictly server tracked
            secure: process.env.NODE_ENV === 'production'
        });
        res.json({ success: true, message: 'Logged in successfully' });
    } else {
        res.status(401).json({ error: 'Invalid password' });
    }
});

// Update Password Route (Internal UI Configuration)
app.post('/api/auth/update_password', (req, res) => {
    const { newPassword } = req.body;
    if (!newPassword || newPassword.length < 3) {
        return res.status(400).json({ error: '新密码无效，至少需要3个字符' });
    }

    // In-memory overwrite of password globally
    ADMIN_PASSWORD = newPassword;

    // Try to physically store it into a local config.json file next to index.js
    const configPath = path.join(__dirname, 'server-config.json');
    try {
        let config = {};
        if (fs.existsSync(configPath)) {
            config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
        }
        config.ADMIN_PASSWORD = newPassword;
        fs.writeFileSync(configPath, JSON.stringify(config, null, 2), 'utf8');
    } catch (err) {
        console.error("Failed to persist password change to disk:", err);
    }

    // Expire current cookies so the user has to login
    res.clearCookie('auth_token');
    res.json({ success: true, message: 'Password updated successfully' });
});

// Logout Route
app.post('/api/auth/logout', (req, res) => {
    res.clearCookie('auth_token');
    res.json({ success: true });
});

// 健康检查
app.get('/api/health', (req, res) => {
    res.json({ status: 'healthy', version: '1.0.0 (JS Node)' });
});

// 进程控制
app.post('/api/process/control', (req, res) => {
    const { action } = req.body;

    if (action === 'start') {
        if (processState) {
            return res.status(400).json({ error: '进程已在运行' });
        }

        const dataDir = getDdbotDataDir();
        const exePath = findDdbotExecutable(dataDir);

        if (!exePath) {
            return res.status(404).json({ error: 'ddbot executable not found in data directory. Please install it first.' });
        }

        try {
            processState = spawn(exePath, [], {
                cwd: dataDir,
                stdio: 'ignore', // 忽略标准输入输出
                detached: false   // 跟随父进程退出
            });

            processState.on('exit', () => {
                processState = null;
            });

            return res.json({ running: true, pid: processState.pid, status: 'running' });
        } catch (error) {
            return res.status(500).json({ error: `启动失败: ${error.message}` });
        }
    }
    else if (action === 'stop') {
        if (processState) {
            processState.kill();
            processState = null;
        }
        return res.json({ running: false, pid: null, status: 'stopped' });
    }
    else if (action === 'restart') {
        if (processState) {
            processState.kill();
            processState = null;
        }

        const dataDir = getDdbotDataDir();
        const exePath = findDdbotExecutable(dataDir);

        if (!exePath) {
            return res.status(404).json({ error: 'ddbot executable not found in data directory. Please install it first.' });
        }

        try {
            processState = spawn(exePath, [], {
                cwd: dataDir,
                stdio: 'ignore'
            });

            processState.on('exit', () => {
                processState = null;
            });

            return res.json({ running: true, pid: processState.pid, status: 'running' });
        } catch (error) {
            return res.status(500).json({ error: `重新启动失败: ${error.message}` });
        }
    }

    return res.status(400).json({ error: '无效操作' });
});

// 进程状态检查
app.get('/api/process/status', (req, res) => {
    if (processState && !processState.killed) {
        return res.json({
            running: true,
            pid: processState.pid,
            status: 'running'
        });
    }
    return res.json({
        running: false,
        pid: null,
        status: 'stopped'
    });
});

// OneBot 状态（占位）
app.get('/api/onebot/status', (req, res) => {
    res.json({
        online: false,
        good: false,
        connected: false,
        protocol: 'OneBot v11',
        self_id: null
    });
});

// 获取订阅概览数据（SQLite）
app.get('/api/subs/summary', (req, res) => {
    const dbPath = path.join(getDdbotDataDir(), '.lsp.db');
    if (!fs.existsSync(dbPath)) {
        return res.json({
            total: 0, active: 0, paused: 0, bySite: {}, offline: true
        });
    }

    try {
        const db = betterSqlite3(dbPath, { readonly: true });
        const rows = db.prepare('SELECT site, COUNT(*) as count FROM subs GROUP BY site').all();
        db.close();

        const bySite = {};
        let total = 0;

        for (const row of rows) {
            bySite[row.site] = row.count;
            total += row.count;
        }

        return res.json({
            total, active: total, paused: 0, bySite, offline: true
        });
    } catch (error) {
        console.error('Error reading sqlite:', error);
        return res.json({
            total: 0, active: 0, paused: 0, bySite: {}, offline: true
        });
    }
});

// 获取配置文件
app.get('/api/config', async (req, res) => {
    const filename = req.query.filename || 'application.yaml';
    const filePath = path.join(getDdbotDataDir(), filename);

    if (!fs.existsSync(filePath)) {
        return res.json({ content: '' });
    }

    try {
        const content = await fs.readFile(filePath, 'utf-8');
        res.json({ content });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// 保存配置文件
app.post('/api/config', async (req, res) => {
    const { filename, content } = req.body;
    if (!filename || !content) {
        return res.status(400).json({ error: 'Missing filename or content' });
    }

    const dataDir = getDdbotDataDir();
    const filePath = path.join(dataDir, filename);
    const backupsDir = getBackupsDir();

    try {
        await fs.ensureDir(dataDir);

        // 创建备份
        if (fs.existsSync(filePath)) {
            await fs.ensureDir(backupsDir);
            const timestamp = dayjs().format('YYYYMMDD_HHmmss');
            const backupName = `${filename}.${timestamp}.bak`;
            await fs.copy(filePath, path.join(backupsDir, backupName));
        }

        await fs.writeFile(filePath, content, 'utf-8');
        res.json({ success: true });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// 获取日志
app.get('/api/logs', async (req, res) => {
    const limit = parseInt(req.query.lines, 10) || 100;
    const level = (req.query.level || '').toUpperCase();

    const logsDir = path.join(getDdbotDataDir(), 'logs');
    if (!fs.existsSync(logsDir)) {
        return res.json({ logs: [] });
    }

    try {
        const files = fs.readdirSync(logsDir)
            .filter(f => f.endsWith('.log'))
            .sort();

        if (files.length === 0) {
            return res.json({ logs: [] });
        }

        const latestLog = files[files.length - 1];
        const logContent = await fs.readFile(path.join(logsDir, latestLog), 'utf-8');

        let lines = logContent.split('\n');

        if (level && level !== 'ALL') {
            lines = lines.filter(line => line.includes(level));
        }

        const start = Math.max(0, lines.length - limit);
        res.json({ logs: lines.slice(start) });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// GitHub Installation API
app.post('/api/install', async (req, res) => {
    try {
        const dataDir = getDdbotDataDir();
        await fs.ensureDir(dataDir);

        console.log("Fetching latest DDBOT-WSa release info...");
        const releaseUrl = 'https://api.github.com/repos/cnxysoft/DDBOT-WSa/releases/latest';
        const response = await axios.get(releaseUrl, {
            headers: { 'User-Agent': 'DDBOT-WSa-Desktop-JS-Backend' }
        });

        const assets = response.data.assets;

        // Define Arch & Platform mapping
        const platformMap = {
            'win32': 'windows',
            'darwin': 'darwin',
            'linux': 'linux'
        };
        const archMap = {
            'x64': ['amd64'],
            'ia32': ['386'],
            'arm64': ['arm64', 'arm'],
            'arm': ['arm']
        };

        const osKw = platformMap[process.platform] || process.platform;
        const archKws = archMap[process.arch] || [process.arch];

        let targetAsset = null;
        for (const archKw of archKws) {
            const regex = new RegExp(`DDBOT-WSa-.*-${osKw}-${archKw}\\.(zip|tar\\.gz)`, 'i');
            targetAsset = assets.find(a => regex.test(a.name));
            if (targetAsset) break;
        }

        if (!targetAsset) {
            return res.status(404).json({ error: `未找到匹配当前系统的 release: ${osKw}-${archKws.join('|')}` });
        }

        console.log(`Downloading ${targetAsset.browser_download_url}...`);

        // Setup temporary file path
        const tempFilePath = path.join(dataDir, targetAsset.name);
        const writer = fs.createWriteStream(tempFilePath);
        const downloadResponse = await axios({
            url: targetAsset.browser_download_url,
            method: 'GET',
            responseType: 'stream',
            headers: { 'User-Agent': 'DDBOT-WSa-Desktop-JS-Backend' }
        });

        downloadResponse.data.pipe(writer);

        await new Promise((resolve, reject) => {
            writer.on('finish', resolve);
            writer.on('error', reject);
        });

        console.log(`Extracting ${tempFilePath}...`);
        const isWin = process.platform === 'win32';
        const targetFilename = isWin ? 'ddbot.exe' : 'ddbot';
        let extractedPath = path.join(dataDir, targetFilename);

        if (targetAsset.name.toLowerCase().endsWith('.zip')) {
            const zip = new AdmZip(tempFilePath);
            const zipEntries = zip.getEntries();

            let found = false;
            for (const entry of zipEntries) {
                if (entry.entryName.toLowerCase().endsWith('ddbot.exe') || entry.entryName.toLowerCase().endsWith('ddbot')) {
                    const content = entry.getData();
                    await fs.writeFile(extractedPath, content);
                    found = true;
                    break;
                }
            }
            if (!found) throw new Error("Zip 中未找到 ddbot 可执行文件!");
        } else if (targetAsset.name.toLowerCase().endsWith('.tar.gz')) {
            // Need to exact tar
            await tar.x({
                file: tempFilePath,
                cwd: dataDir,
                filter: (p) => p.toLowerCase().endsWith('ddbot') || p.toLowerCase().endsWith('ddbot.exe'),
                strip: 1 // strip top level dir
            });
            // rename just in case
            const potentiallyExtracted = path.join(dataDir, targetFilename);
            if (!fs.existsSync(potentiallyExtracted) && fs.existsSync(path.join(dataDir, 'ddbot-wsa'))) {
                fs.renameSync(path.join(dataDir, 'ddbot-wsa'), potentiallyExtracted);
            }
        }

        // Set permissions on unix
        if (!isWin && fs.existsSync(extractedPath)) {
            await fs.chmod(extractedPath, '755');
        }

        // Cleanup temp file
        await fs.unlink(tempFilePath);
        console.log("Install complete!");

        res.json({ success: true, message: `已安装至 ${extractedPath}` });

    } catch (error) {
        console.error("Install Error:", error);
        res.status(500).json({ error: `安装失败: ${error.message}` });
    }
});

// 处理 Vue Router 的 HTML5 History 模式回退
// 当请求的路径不是 /api 开头，且在 dist 里找不到文件时，统一返回 index.html
app.use((req, res, next) => {
    if (req.method !== 'GET' || req.path.startsWith('/api')) {
        return next();
    }

    const indexPath = path.join(distPath, 'index.html');
    if (fs.existsSync(indexPath)) {
        res.sendFile(indexPath);
    } else {
        next();
    }
});

// 404 Not Found Handler for unrecognized API routes (prevents strict CSP headers from Express default reporter)
app.use('/api', (req, res) => {
    res.status(404).json({ error: 'Not Found' });
});

app.listen(PORT, () => {
    console.log(`DDBot Node.js Server running on port ${PORT}`);
});
