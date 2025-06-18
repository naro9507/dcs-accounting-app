const { app, BrowserWindow } = require('electron')
const { autoUpdater } = require('electron-updater')
const isDev = process.env.NODE_ENV === 'development'
const path = require('path')
const pino = require('pino')

// 本番環境で環境変数を読み込み
if (!isDev) {
  require('dotenv').config({ path: path.join(__dirname, '../.env.production') })
} else {
  require('dotenv').config({ path: path.join(__dirname, '../.env.development') })
}

const logger = pino({
  level: isDev ? 'debug' : 'info',
  timestamp: pino.stdTimeFunctions.isoTime,
  formatters: {
    level: (label) => ({ level: label.toUpperCase() }),
  },
  transport: isDev ? {
    target: 'pino-pretty',
    options: {
      colorize: true,
      ignore: 'pid,hostname',
      translateTime: 'yyyy-mm-dd HH:MM:ss',
    }
  } : undefined
})

autoUpdater.logger = logger

if (!isDev) {
  autoUpdater.checkForUpdatesAndNotify()
}

autoUpdater.on('checking-for-update', () => {
  logger.info('Checking for update...')
})

autoUpdater.on('update-available', (info) => {
  logger.info('Update available', info)
})

autoUpdater.on('update-not-available', (info) => {
  logger.info('Update not available', info)
})

autoUpdater.on('error', (err) => {
  logger.error('Error in auto-updater', err)
})

autoUpdater.on('download-progress', (progressObj) => {
  logger.info('Download progress', {
    bytesPerSecond: progressObj.bytesPerSecond,
    percent: progressObj.percent,
    transferred: progressObj.transferred,
    total: progressObj.total
  })
})

autoUpdater.on('update-downloaded', (info) => {
  logger.info('Update downloaded', info)
  autoUpdater.quitAndInstall()
})

function createWindow() {
  logger.info('Creating main window')
  
  const mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false
    }
  })

  if (isDev) {
    logger.debug('Loading development URL')
    mainWindow.loadURL('http://localhost:3000')
    mainWindow.webContents.openDevTools()
  } else {
    logger.debug('Loading production HTML file')
    mainWindow.loadFile(path.join(__dirname, '../dist/ja/index.html'))
  }
}

app.whenReady().then(() => {
  logger.info('App is ready')
  createWindow()
})

app.on('window-all-closed', () => {
  logger.info('All windows closed')
  if (process.platform !== 'darwin') {
    logger.info('Quitting app')
    app.quit()
  }
})

app.on('activate', () => {
  logger.info('App activated')
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow()
  }
})