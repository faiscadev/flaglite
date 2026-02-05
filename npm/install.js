#!/usr/bin/env node
/**
 * Post-install script for @faiscadev/flaglite-cli
 * Downloads the appropriate binary for the current platform
 */

const https = require('https');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const REPO = 'faiscadev/flaglite';
const BINARY_NAME = 'flaglite';

// Map Node platform/arch to our binary names
function getBinaryName() {
  const platform = process.platform;
  const arch = process.arch;

  const platformMap = {
    darwin: 'macos',
    linux: 'linux',
  };

  const archMap = {
    x64: 'amd64',
    arm64: 'arm64',
  };

  const os = platformMap[platform];
  const cpu = archMap[arch];

  if (!os || !cpu) {
    throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }

  return `${BINARY_NAME}-${os}-${cpu}`;
}

// Get the latest release version
async function getLatestVersion() {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: 'api.github.com',
      path: `/repos/${REPO}/releases/latest`,
      headers: {
        'User-Agent': 'flaglite-npm-installer',
      },
    };

    https.get(options, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        try {
          const release = JSON.parse(data);
          // Extract version from tag like "cli-v0.1.0" -> "0.1.0"
          const version = release.tag_name.replace('cli-v', '');
          resolve(version);
        } catch (e) {
          reject(new Error('Failed to parse release info'));
        }
      });
    }).on('error', reject);
  });
}

// Download a file
async function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    
    const request = (url) => {
      https.get(url, { headers: { 'User-Agent': 'flaglite-npm-installer' } }, (res) => {
        // Handle redirects
        if (res.statusCode === 302 || res.statusCode === 301) {
          request(res.headers.location);
          return;
        }
        
        if (res.statusCode !== 200) {
          reject(new Error(`Download failed with status ${res.statusCode}`));
          return;
        }
        
        res.pipe(file);
        file.on('finish', () => {
          file.close(resolve);
        });
      }).on('error', (err) => {
        fs.unlink(dest, () => {});
        reject(err);
      });
    };
    
    request(url);
  });
}

async function install() {
  const binDir = path.join(__dirname, 'bin');
  const binPath = path.join(binDir, BINARY_NAME);

  // Skip if already installed
  if (fs.existsSync(binPath)) {
    console.log('FlagLite CLI already installed');
    return;
  }

  try {
    const binaryName = getBinaryName();
    console.log(`Installing FlagLite CLI for ${process.platform}-${process.arch}...`);

    // Get version from package.json or latest release
    const pkg = require('./package.json');
    const version = pkg.version;

    const downloadUrl = `https://github.com/${REPO}/releases/download/cli-v${version}/${binaryName}`;
    
    console.log(`Downloading from ${downloadUrl}...`);

    // Create bin directory
    fs.mkdirSync(binDir, { recursive: true });

    // Download binary
    await download(downloadUrl, binPath);

    // Make executable
    fs.chmodSync(binPath, 0o755);

    console.log('FlagLite CLI installed successfully!');
    console.log('Run `flaglite --help` to get started.');
  } catch (error) {
    console.error('Failed to install FlagLite CLI:', error.message);
    console.error('You can install manually: curl -fsSL https://flaglite.dev/install.sh | sh');
    process.exit(1);
  }
}

install();
