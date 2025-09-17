// Main application entry point
import { appState } from './state.js';
import { disableAutocomplete } from './autocomplete.js';
import { setupEventListeners, loadSettingsAndUpdateUI } from './events.js';
import { updateUI } from './ui.js';

// Initialize the application
async function initApp() {
    console.log('Initializing StreamFlow-Tauri...');

    try {
        // Load settings from backend
        await loadSettingsAndUpdateUI();

        // Setup event listeners
        setupEventListeners();

        // Disable autocomplete on all inputs
        disableAutocomplete();

        // Update UI
        await updateUI();

        console.log('Application initialized successfully');
    } catch (error) {
        console.error('Failed to initialize app:', error);
        // Use a simple alert since we don't have access to showMessage yet
        alert('Failed to initialize application: ' + error);
    }
}

// Handle app focus/blur for status updates
window.addEventListener('focus', updateUI);

// Periodic status checks
setInterval(() => {
    if (appState.isStreaming) {
        // Import the check function dynamically or move it to a better place
        import('./events.js').then(module => {
            module.checkVlcStatus();
        });
    }
}, 30000); // Check every 30 seconds

// Initialize app when DOM is loaded
document.addEventListener('DOMContentLoaded', initApp);