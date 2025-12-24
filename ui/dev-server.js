#!/usr/bin/env node

// Simple HTTP server for Tauri dev mode
const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = 1420;
const DIR = __dirname;

const MIME_TYPES = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.png': 'image/png',
  '.jpg': 'image/jpg',
  '.gif': 'image/gif',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon'
};

const server = http.createServer((req, res) => {
  let filePath = path.join(DIR, req.url === '/' ? 'index.html' : req.url);
  
  const extname = String(path.extname(filePath)).toLowerCase();
  const contentType = MIME_TYPES[extname] || 'application/octet-stream';

  fs.readFile(filePath, (error, content) => {
    if (error) {
      if (error.code === 'ENOENT') {
        // If file not found, try index.html
        fs.readFile(path.join(DIR, 'index.html'), (err, content) => {
          if (err) {
            res.writeHead(500);
            res.end('Server Error: ' + err.code);
          } else {
            res.writeHead(200, { 'Content-Type': 'text/html' });
            res.end(content, 'utf-8');
          }
        });
      } else {
        res.writeHead(500);
        res.end('Server Error: ' + error.code);
      }
    } else {
      res.writeHead(200, { 'Content-Type': contentType });
      res.end(content, 'utf-8');
    }
  });
});

server.listen(PORT, (err) => {
  if (err) {
    if (err.code === 'EADDRINUSE') {
      console.error(`Port ${PORT} is already in use. Trying to kill existing process...`);
      // Try to find and kill process using this port
      const { exec } = require('child_process');
      exec(`lsof -ti:${PORT} | xargs kill -9 2>/dev/null`, (error) => {
        if (error) {
          console.error(`Could not kill process on port ${PORT}. Please kill it manually.`);
          process.exit(1);
        } else {
          console.log(`Killed process on port ${PORT}, retrying...`);
          setTimeout(() => {
            server.listen(PORT, () => {
              console.log(`Server running at http://localhost:${PORT}/`);
            });
          }, 1000);
        }
      });
    } else {
      console.error(`Server error: ${err.message}`);
      process.exit(1);
    }
  } else {
    console.log(`Server running at http://localhost:${PORT}/`);
  }
});

// Keep process alive
process.on('SIGINT', () => {
  console.log('\nShutting down server...');
  server.close(() => {
    process.exit(0);
  });
});

