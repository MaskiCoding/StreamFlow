// Simple JS bundler - combines ES6 modules into single file for compatibility
const fs = require('fs');
const path = require('path');

function buildJS() {
    console.log('Building JavaScript modules...');

    // Read all module files
    const modules = {
        'state.js': fs.readFileSync(path.join(__dirname, 'js/state.js'), 'utf8'),
        'utils.js': fs.readFileSync(path.join(__dirname, 'js/utils.js'), 'utf8'),
        'autocomplete.js': fs.readFileSync(path.join(__dirname, 'js/autocomplete.js'), 'utf8'),
        'api.js': fs.readFileSync(path.join(__dirname, 'js/api.js'), 'utf8'),
        'ui.js': fs.readFileSync(path.join(__dirname, 'js/ui.js'), 'utf8'),
        'events.js': fs.readFileSync(path.join(__dirname, 'js/events.js'), 'utf8'),
        'main.js': fs.readFileSync(path.join(__dirname, 'js/main.js'), 'utf8')
    };

    // Convert ES6 modules to IIFE (Immediately Invoked Function Expression) format
    let bundled = `// Bundled JavaScript - Generated from ES6 modules
// Tauri API imports
const { invoke } = window.__TAURI__.core;

// Global state and elements (converted from modules)
`;

    // Convert state module
    bundled += modules['state.js']
        .replace(/export { appState, elements };/, '')
        .replace(/export/g, '');

    // Convert utils module
    bundled += '\n// Utility functions\n';
    bundled += modules['utils.js']
        .replace(/export /g, '')
        .replace(/import {.*} from '\.\/utils\.js';/g, '');

    // Convert autocomplete module
    bundled += '\n// Autocomplete functions\n';
    bundled += modules['autocomplete.js']
        .replace(/export /g, '');

    // Convert API module
    bundled += '\n// API functions\n';
    bundled += modules['api.js']
        .replace(/export /g, '');

    // Convert UI module
    bundled += '\n// UI functions\n';
    bundled += modules['ui.js']
        .replace(/import { appState, elements } from '\.\/state\.js';/g, '')
        .replace(/import { showMessage } from '\.\/utils\.js';/g, '')
        .replace(/export /g, '');

    // Convert events module
    bundled += '\n// Event handlers\n';
    bundled += modules['events.js']
        .replace(/import { appState, elements } from '\.\/state\.js';/g, '')
        .replace(/import { showMessage, getErrorMessage } from '\.\/utils\.js';/g, '')
        .replace(/import { updateUI, setStartingState, showAddStreamModal, hideAddStreamModal, showContextMenu, hideContextMenu } from '\.\/ui\.js';/g, '')
        .replace(/import { validateTwitchUrl, normalizeTwitchUrl, loadSettings, saveSettings, addQuickStream, removeQuickStream, updateQuickStreamStatus, startStream as apiStartStream, stopStream as apiStopStream, extractChannelName, isVlcRunning } from '\.\/api\.js';/g, '')
        .replace(/export /g, '');

    // Convert main module
    bundled += '\n// Main application\n';
    bundled += modules['main.js']
        .replace(/import { appState } from '\.\/state\.js';/g, '')
        .replace(/import { disableAutocomplete } from '\.\/autocomplete\.js';/g, '')
        .replace(/import { setupEventListeners, loadSettingsAndUpdateUI } from '\.\/events\.js';/g, '')
        .replace(/import { updateUI } from '\.\/ui\.js';/g, '');

    // Write the bundled file
    fs.writeFileSync(path.join(__dirname, 'script.js'), bundled);
    console.log('✅ JavaScript bundle created successfully!');
}

if (require.main === module) {
    buildJS();
}

module.exports = { buildJS };