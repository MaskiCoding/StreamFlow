// Tauri API imports
const { invoke } = window.__TAURI__.core;

// Application state
let appState = {
    isStreaming: false,
    currentUrl: '',
    selectedQuality: 'best',
    quickStreams: [],
    isStartingStream: false
};

// DOM elements
const elements = {
    urlInput: document.getElementById('url-input'),
    qualitySelect: document.getElementById('quality-select'),
    mainActionBtn: document.getElementById('main-action-btn'),
    statusDot: document.getElementById('status-dot'),
    statusText: document.getElementById('status-text'),
    messageDisplay: document.getElementById('message-display'),
    contextMenu: document.getElementById('context-menu'),
    addStreamModal: document.getElementById('add-stream-modal'),
    streamName: document.getElementById('stream-name'),
    streamUrl: document.getElementById('stream-url'),
    quickStreamSlots: document.querySelectorAll('.quick-stream-slot')
};

// Initialize the application
async function initApp() {
    console.log('Initializing StreamFlow-Tauri...');
    
    try {
        // Load settings from backend
        await loadSettings();
        
        // Setup event listeners
        setupEventListeners();
        
        // Update UI
        await updateUI();
        
        console.log('Application initialized successfully');
        showMessage('StreamFlow ready!', 'success');
    } catch (error) {
        console.error('Failed to initialize app:', error);
        showMessage('Failed to initialize application: ' + error, 'error');
    }
}

// Setup all event listeners
function setupEventListeners() {
    // URL input events
    elements.urlInput.addEventListener('input', handleUrlInput);
    elements.urlInput.addEventListener('keypress', handleUrlKeypress);
    
    // Quality selection
    elements.qualitySelect.addEventListener('change', handleQualityChange);
    
    // Main action button
    elements.mainActionBtn.addEventListener('click', handleMainAction);
    
    // Quick stream slots
    elements.quickStreamSlots.forEach((slot, index) => {
        slot.addEventListener('click', () => handleQuickStreamClick(index));
        slot.addEventListener('contextmenu', (e) => handleQuickStreamRightClick(e, index));
    });
    
    // Context menu
    document.addEventListener('click', hideContextMenu);
    elements.contextMenu.addEventListener('click', handleContextMenuClick);
    
    // Refresh status button
    document.getElementById('refresh-status-btn').addEventListener('click', refreshAllStreamStatus);
    
    // Modal events
    document.getElementById('cancel-add-stream').addEventListener('click', hideAddStreamModal);
    document.getElementById('confirm-add-stream').addEventListener('click', handleAddStream);
    
    // Close modal on outside click
    elements.addStreamModal.addEventListener('click', (e) => {
        if (e.target === elements.addStreamModal) {
            hideAddStreamModal();
        }
    });
    
    // Keyboard shortcuts
    document.addEventListener('keydown', handleKeydown);
}

// Handle URL input changes
async function handleUrlInput() {
    const url = elements.urlInput.value.trim();
    
    if (url) {
        try {
            const isValid = await invoke('validate_twitch_url', { url });
            elements.urlInput.style.borderColor = isValid ? 'var(--iris)' : 'var(--love)';
        } catch (error) {
            console.error('URL validation error:', error);
        }
    } else {
        elements.urlInput.style.borderColor = 'var(--overlay)';
    }
}

// Handle Enter key in URL input
function handleUrlKeypress(e) {
    if (e.key === 'Enter' && !appState.isStreaming && !appState.isStartingStream) {
        handleMainAction();
    }
}

// Handle quality selection change
async function handleQualityChange() {
    appState.selectedQuality = elements.qualitySelect.value;
    
    try {
        // Save quality preference to backend
        const settings = await invoke('load_settings');
        settings.default_quality = appState.selectedQuality;
        await invoke('save_settings', { newSettings: settings });
    } catch (error) {
        console.error('Failed to save quality setting:', error);
    }
}

// Handle main action button (Watch/Stop Stream)
async function handleMainAction() {
    if (appState.isStartingStream) return;
    
    if (appState.isStreaming) {
        await stopStream();
    } else {
        const url = elements.urlInput.value.trim();
        // Check if current stream status is offline
        let isOffline = false;
        if (appState.quickStreams && url) {
            const found = appState.quickStreams.find(s => s && s.url === url);
            if (found && found.status === 'Offline') {
                isOffline = true;
            }
        }
        if (isOffline) {
            showMessage('This stream is offline and cannot be started.', 'error');
            return;
        }
        if (url) {
            await startStream(url, appState.selectedQuality);
        } else {
            showMessage('Please enter a Twitch URL or username', 'error');
        }
    }
}

// Start streaming
async function startStream(url, quality) {
    console.log('Starting stream:', url, 'quality:', quality);
    
    try {
        // Normalize URL
        console.log('Normalizing URL:', url);
        const normalizedUrl = await invoke('normalize_twitch_url', { url });
        console.log('Normalized URL:', normalizedUrl);
        
        // Update UI to starting state
        setStartingState(true);
        
        // Start stream via backend
        const result = await invoke('start_stream', { 
            url: normalizedUrl, 
            quality 
        });
        
        console.log('Stream started:', result);
        
        // Update state
        appState.isStreaming = true;
        appState.currentUrl = normalizedUrl;
        
        // Update UI
        await updateUI();
        
        showMessage('Stream started successfully!', 'success');
        
        // Check VLC status after a delay
        setTimeout(checkVlcStatus, 3000);
        
    } catch (error) {
        console.error('Failed to start stream:', error);
        // Enhanced error handling with specific messages using shared function
        showMessage(getErrorMessage(error), 'error');
    } finally {
        setStartingState(false);
    }
}

// Stop streaming
async function stopStream() {
    console.log('Stopping stream');
    
    try {
        const result = await invoke('stop_stream');
        console.log('Stream stopped:', result);
        
        // Update state
        appState.isStreaming = false;
        appState.currentUrl = '';
        
        // Update UI
        await updateUI();
        
        showMessage('Stream stopped', 'success');
        
    } catch (error) {
        console.error('Failed to stop stream:', error);
        // Enhanced error handling for stopping streams using shared function
        showMessage(getErrorMessage(error), 'error');
    }
}

// Set starting state
function setStartingState(starting) {
    appState.isStartingStream = starting;
    updateMainButton();
    updateStatusIndicator();
}

// Check VLC status
async function checkVlcStatus() {
    try {
        const isVlcRunning = await invoke('is_vlc_running');
        if (!isVlcRunning && appState.isStreaming) {
            showMessage('VLC not detected. Make sure VLC is installed and set as default player.', 'error');
        }
    } catch (error) {
        console.error('Failed to check VLC status:', error);
    }
}

// Shared function to get user-friendly error messages
function getErrorMessage(error) {
    if (error.includes('HTTP client initialization failed')) {
        return 'Network error: Could not initialize connection to Twitch. Please check your internet connection.';
    } else if (error.includes('Network error reading response')) {
        return 'Network error: Failed to read response from Twitch. Please try again.';
    } else if (error.includes('Invalid Twitch URL')) {
        return 'Invalid URL: Please enter a valid Twitch channel URL or username.';
    } else if (error.includes('Streamlink not found')) {
        return 'Streamlink not found: Please ensure Streamlink is installed and in your PATH.';
    } else if (error.includes('VLC not found')) {
        return 'VLC not found: Please ensure VLC is installed and in your PATH.';
    } else if (error.includes('Failed to check stream status')) {
        return 'Status check failed: Unable to determine stream status. Please try again.';
    } else if (error.includes('Failed to parse HTML selector')) {
        return 'Parsing error: Could not analyze stream page. Please try again.';
    } else if (error.includes('No stream to stop')) {
        return 'No active stream to stop.';
    } else if (error.includes('Process termination failed')) {
        return 'Failed to terminate stream process. Please close VLC manually if needed.';
    }
    return 'An unexpected error occurred: ' + error;
}

// Handle quick stream slot click
async function handleQuickStreamClick(index) {
    const slot = elements.quickStreamSlots[index];
    const streamButton = slot.querySelector('.stream-button');
    
    if (streamButton) {
        // Load existing stream and switch to it
        const streamData = appState.quickStreams[index];
        if (streamData) {
            if (streamData.status === 'Offline') {
                showMessage('This stream is offline and cannot be started.', 'error');
                return;
            }
            // Stop current stream first if one is running
            if (appState.isStreaming) {
                console.log('Stopping current stream before switching...');
                await stopStream();
                // Wait a bit for the stop to complete
                await new Promise(resolve => setTimeout(resolve, 1000));
            }
            
            // Set the new URL and start the stream
            elements.urlInput.value = streamData.url;
            console.log('Switching to stream:', streamData.url);
            await startStream(streamData.url, appState.selectedQuality);
        }
    } else {
        // Show add stream modal
        showAddStreamModal(index);
    }
}

// Handle quick stream right-click
function handleQuickStreamRightClick(e, index) {
    e.preventDefault();
    
    const slot = elements.quickStreamSlots[index];
    const streamButton = slot.querySelector('.stream-button');
    
    if (streamButton) {
        showContextMenu(e.clientX, e.clientY, index);
    }
}

// Show context menu
function showContextMenu(x, y, index) {
    elements.contextMenu.style.left = x + 'px';
    elements.contextMenu.style.top = y + 'px';
    elements.contextMenu.style.display = 'block';
    elements.contextMenu.dataset.index = index;
}

// Hide context menu
function hideContextMenu() {
    elements.contextMenu.style.display = 'none';
}

// Handle context menu click
async function handleContextMenuClick(e) {
    const action = e.target.dataset.action;
    const index = parseInt(elements.contextMenu.dataset.index);
    
    if (action === 'remove') {
        await removeQuickStream(index);
    }
    
    hideContextMenu();
}

// Refresh status of all quick streams
async function refreshAllStreamStatus() {
    const refreshBtn = document.getElementById('refresh-status-btn');
    const originalText = refreshBtn.textContent;
    
    refreshBtn.disabled = true;
    refreshBtn.classList.add('checking');
    refreshBtn.textContent = 'Checking...';
    
    try {
        // Update all stream statuses
        const result = await invoke('update_quick_stream_status');
        
        // Reload settings and update UI to show new statuses
        await loadSettings();
        await updateUI();
        
        showMessage(result || 'Stream statuses updated!', 'success');
    } catch (error) {
        console.error('Failed to refresh stream status:', error);
        showMessage('Failed to refresh stream statuses: ' + error, 'error');
    } finally {
        refreshBtn.disabled = false;
        refreshBtn.classList.remove('checking');
        refreshBtn.textContent = originalText;
    }
}

// Show add stream modal
function showAddStreamModal(index) {
    elements.addStreamModal.classList.add('show');
    elements.addStreamModal.dataset.index = index;
    elements.streamName.value = '';
    elements.streamUrl.value = '';
    elements.streamName.focus();
}

// Hide add stream modal
function hideAddStreamModal() {
    elements.addStreamModal.classList.remove('show');
}

// Handle add stream
async function handleAddStream() {
    const name = elements.streamName.value.trim();
    let input = elements.streamUrl.value.trim();
    
    if (!name || !input) {
        showMessage('Please fill in both name and stream input', 'error');
        return;
    }
    
    try {
        // If input doesn't contain twitch.tv or https, treat it as a username
        let url = input;
        if (!input.includes('twitch.tv') && !input.includes('https://')) {
            // Just a username, convert to full URL
            url = `https://www.twitch.tv/${input}`;
        }
        
        // Validate URL
        const isValid = await invoke('validate_twitch_url', { url });
        if (!isValid) {
            showMessage('Invalid Twitch username or URL', 'error');
            return;
        }
        
        // Normalize URL
        const normalizedUrl = await invoke('normalize_twitch_url', { url });
        
        // Check for duplicates
        const isDuplicate = appState.quickStreams.some(stream =>
            stream && stream.url === normalizedUrl
        );
        
        if (isDuplicate) {
            showMessage('This stream is already in your quick streams!', 'error');
            return;
        }
        
        // Add to backend
        await invoke('add_quick_stream', {
            name,
            url: normalizedUrl
        });
        
        // Reload settings and update UI
        await loadSettings();
        await updateUI();
        
        hideAddStreamModal();
        showMessage('Quick stream added!', 'success');
        
    } catch (error) {
        console.error('Failed to add quick stream:', error);
        // Enhanced error handling with specific messages using shared function
        showMessage(getErrorMessage(error), 'error');
    }
}

// Remove quick stream
async function removeQuickStream(index) {
    try {
        await invoke('remove_quick_stream', { index });
        
        // Reload settings and update UI
        await loadSettings();
        await updateUI();
        
        showMessage('Quick stream removed', 'success');
        
    } catch (error) {
        console.error('Failed to remove quick stream:', error);
        showMessage('Failed to remove quick stream: ' + error, 'error');
    }
}

// Load settings from backend
async function loadSettings() {
    try {
        const settings = await invoke('load_settings');
        
        appState.selectedQuality = settings.default_quality || 'best';
        appState.quickStreams = settings.quick_streams || [];
        
        // Update quality selector
        elements.qualitySelect.value = appState.selectedQuality;
        
    } catch (error) {
        console.error('Failed to load settings:', error);
        throw error;
    }
}

// Update UI based on current state
async function updateUI() {
    updateMainButton();
    updateStatusIndicator();
    updateQuickStreams();
    
    // Check current stream status
    try {
        const currentStream = await invoke('get_current_stream');
        if (currentStream) {
            appState.isStreaming = true;
            appState.currentUrl = currentStream;
        }
    } catch (error) {
        console.error('Failed to get current stream:', error);
    }
}

// Update main action button
function updateMainButton() {
    const btn = elements.mainActionBtn;
    const icon = btn.querySelector('.btn-icon');
    const text = btn.querySelector('.btn-text');
    if (appState.isStartingStream) {
        btn.disabled = true;
        btn.classList.remove('streaming');
        icon.textContent = '⟳';
        text.textContent = 'Starting...';
    } else if (appState.isStreaming) {
        btn.disabled = false;
        btn.classList.add('streaming');
        icon.textContent = '⏹';
        text.textContent = 'Stop Stream';
    } else {
        // Check if current stream status is offline
        let isOffline = false;
        const url = elements.urlInput.value.trim();
        if (appState.quickStreams && url) {
            const found = appState.quickStreams.find(s => s && s.url === url);
            if (found && found.status === 'Offline') {
                isOffline = true;
            }
        }
        btn.disabled = isOffline;
        btn.classList.remove('streaming');
        icon.textContent = '▶';
        text.textContent = isOffline ? 'Offline' : 'Watch Stream';
    }
}

// Update status indicator
function updateStatusIndicator() {
    const dot = elements.statusDot;
    const text = elements.statusText;
    
    dot.className = 'status-dot';
    
    if (appState.isStartingStream) {
        dot.classList.add('starting');
        text.textContent = 'Starting stream...';
    } else if (appState.isStreaming) {
        dot.classList.add('streaming');
        
        // Extract channel name for display
        if (appState.currentUrl) {
            invoke('extract_channel_name', { url: appState.currentUrl })
                .then(channel => {
                    if (channel) {
                        const capitalized = channel.charAt(0).toUpperCase() + channel.slice(1);
                        text.textContent = `Streaming: ${capitalized}`;
                    } else {
                        text.textContent = 'Streaming';
                    }
                })
                .catch(() => {
                    text.textContent = 'Streaming';
                });
        } else {
            text.textContent = 'Streaming';
        }
    } else {
        dot.classList.add('idle');
        text.textContent = 'Idle';
    }
}

// Update quick stream slots
function updateQuickStreams() {
    elements.quickStreamSlots.forEach((slot, index) => {
        const stream = appState.quickStreams[index];
        
        // Clear existing content
        slot.innerHTML = '';
        
        if (stream) {
            // Debug: Log the actual status value
            console.log(`Stream ${index} (${stream.name}): status =`, stream.status, typeof stream.status);
            
            // Create stream button
            const button = document.createElement('div');
            button.className = 'stream-button';
            if (stream.status === 'Offline') {
                button.classList.add('offline');
            }
            
            // Status dot
            const statusDot = document.createElement('div');
            statusDot.className = 'status-dot-small';
            if (stream.status === 'Online') {
                statusDot.classList.add('online');
            } else if (stream.status === 'Offline') {
                statusDot.classList.add('offline');
            } else {
                statusDot.classList.add('unknown');
            }
            
            // Stream name
            const name = document.createElement('div');
            name.className = 'stream-name';
            name.textContent = stream.name;
            
            // Stream status text
            const status = document.createElement('div');
            status.className = 'stream-status';
            status.textContent = stream.status || 'Unknown';
            
            button.appendChild(statusDot);
            button.appendChild(name);
            button.appendChild(status);
            slot.appendChild(button);
        } else {
            // Create empty slot
            const emptySlot = document.createElement('div');
            emptySlot.className = 'empty-slot';
            
            const plusIcon = document.createElement('span');
            plusIcon.className = 'plus-icon';
            plusIcon.textContent = '+';
            
            emptySlot.appendChild(plusIcon);
            slot.appendChild(emptySlot);
        }
    });
}

// Show message to user
function showMessage(message, type = 'info') {
    const messageEl = elements.messageDisplay;

    // Clear any existing content and create new structure
    messageEl.innerHTML = '';

    // Create message text
    const messageText = document.createElement('span');
    messageText.textContent = message;
    messageEl.appendChild(messageText);

    // Add dismiss button for error messages
    if (type === 'error') {
        const dismissBtn = document.createElement('button');
        dismissBtn.textContent = '✕';
        dismissBtn.className = 'message-dismiss';
        dismissBtn.onclick = () => hideMessage();
        dismissBtn.title = 'Dismiss message';
        messageEl.appendChild(dismissBtn);
    }

    // Set base classes
    messageEl.className = 'message-display show';

    // Add type-specific classes
    if (type === 'error') {
        messageEl.classList.add('error');
    } else if (type === 'success') {
        messageEl.classList.add('success');
    }

    // Different auto-hide timing based on message type
    const hideDelay = type === 'error' ? 8000 : 3000; // 8 seconds for errors, 3 for others

    // Auto hide after delay
    setTimeout(() => {
        hideMessage();
    }, hideDelay);
}

function hideMessage() {
    const messageEl = elements.messageDisplay;
    messageEl.classList.remove('show');
    setTimeout(() => {
        messageEl.className = 'message-display';
        messageEl.innerHTML = '';
    }, 300);
}

// Handle keyboard shortcuts
function handleKeydown(e) {
    // Escape key - close modals/menus
    if (e.key === 'Escape') {
        hideAddStreamModal();
        hideContextMenu();
    }
    
    // Ctrl+Enter - Start/stop stream
    if (e.ctrlKey && e.key === 'Enter') {
        handleMainAction();
    }
}

// Initialize app when DOM is loaded
document.addEventListener('DOMContentLoaded', initApp);

// Handle app focus/blur for status updates
window.addEventListener('focus', updateUI);

// Periodic status checks
setInterval(() => {
    if (appState.isStreaming) {
        checkVlcStatus();
    }
}, 30000); // Check every 30 seconds
