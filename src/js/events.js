// Event handlers and user interactions
import { appState, elements } from './state.js';
import { showMessage, getErrorMessage } from './utils.js';
import { updateUI, setStartingState, showAddStreamModal, hideAddStreamModal, showContextMenu, hideContextMenu } from './ui.js';
import { validateTwitchUrl, normalizeTwitchUrl, loadSettings, saveSettings, addQuickStream, removeQuickStream, updateQuickStreamStatus, startStream as apiStartStream, stopStream as apiStopStream, extractChannelName, isVlcRunning } from './api.js';

// Handle URL input changes
export async function handleUrlInput() {
    const url = elements.urlInput.value.trim();

    if (url) {
        try {
            const isValid = await validateTwitchUrl(url);
            elements.urlInput.style.borderColor = isValid ? 'var(--iris)' : 'var(--love)';
        } catch (error) {
            console.error('URL validation error:', error);
        }
    } else {
        elements.urlInput.style.borderColor = 'var(--overlay)';
    }
}

// Handle Enter key in URL input
export function handleUrlKeypress(e) {
    if (e.key === 'Enter' && !appState.isStreaming && !appState.isStartingStream) {
        handleMainAction();
    }
}

// Handle quality selection change
export async function handleQualityChange() {
    appState.selectedQuality = elements.qualitySelect.value;

    try {
        // Save quality preference to backend
        const settings = await loadSettings();
        settings.default_quality = appState.selectedQuality;
        await saveSettings(settings);
    } catch (error) {
        console.error('Failed to save quality setting:', error);
    }
}

// Handle main action button (Watch/Stop Stream)
export async function handleMainAction() {
    if (appState.isStartingStream) return;

    if (appState.isStreaming) {
        await handleStopStream();
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
            await handleStartStream(url, appState.selectedQuality);
        } else {
            showMessage('Please enter a Twitch URL or username', 'error');
        }
    }
}

// Handle starting a stream
export async function handleStartStream(url, quality) {
    console.log('Starting stream:', url, 'quality:', quality);

    try {
        // Normalize URL
        console.log('Normalizing URL:', url);
        const normalizedUrl = await normalizeTwitchUrl(url);
        console.log('Normalized URL:', normalizedUrl);

        // Update UI to starting state
        setStartingState(true);

        // Start stream via backend
        const result = await apiStartStream(normalizedUrl, quality);

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

// Handle stopping a stream
export async function handleStopStream() {
    console.log('Stopping stream');

    try {
        const result = await apiStopStream();
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

// Check VLC status
export async function checkVlcStatus() {
    try {
        const isVlcRunningCheck = await isVlcRunning();
        if (!isVlcRunningCheck && appState.isStreaming) {
            showMessage('VLC not detected. Make sure VLC is installed and set as default player.', 'error');
        }
    } catch (error) {
        console.error('Failed to check VLC status:', error);
    }
}

// Handle quick stream slot click
export async function handleQuickStreamClick(index) {
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
                await handleStopStream();
                // Wait a bit for the stop to complete
                await new Promise(resolve => setTimeout(resolve, 1000));
            }

            // Set the new URL and start the stream
            elements.urlInput.value = streamData.url;
            console.log('Switching to stream:', streamData.url);
            await handleStartStream(streamData.url, appState.selectedQuality);
        }
    } else {
        // Show add stream modal
        showAddStreamModal(index);
    }
}

// Handle quick stream right-click
export function handleQuickStreamRightClick(e, index) {
    e.preventDefault();

    const slot = elements.quickStreamSlots[index];
    const streamButton = slot.querySelector('.stream-button');

    if (streamButton) {
        showContextMenu(e.clientX, e.clientY, index);
    }
}

// Handle context menu click
export async function handleContextMenuClick(e) {
    const action = e.target.dataset.action;
    const index = parseInt(elements.contextMenu.dataset.index);

    if (action === 'remove') {
        await removeQuickStream(index);
        await loadSettingsAndUpdateUI();
    }

    hideContextMenu();
}

// Handle add stream submission
export async function handleAddStream() {
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
        const isValid = await validateTwitchUrl(url);
        if (!isValid) {
            showMessage('Invalid Twitch username or URL', 'error');
            return;
        }

        // Normalize URL
        const normalizedUrl = await normalizeTwitchUrl(url);

        // Check for duplicates
        const isDuplicate = appState.quickStreams.some(stream =>
            stream && stream.url === normalizedUrl
        );

        if (isDuplicate) {
            showMessage('This stream is already in your quick streams!', 'error');
            return;
        }

        // Add to backend
        await addQuickStream(name, normalizedUrl);

        // Reload settings and update UI
        await loadSettingsAndUpdateUI();

        hideAddStreamModal();
        showMessage('Stream saved!', 'success');

    } catch (error) {
        console.error('Failed to add quick stream:', error);
        // Enhanced error handling with specific messages using shared function
        showMessage(getErrorMessage(error), 'error');
    }
}

// Refresh status of all quick streams
export async function refreshAllStreamStatus() {
    const refreshBtn = document.getElementById('refresh-status-btn');
    const originalText = refreshBtn.textContent;

    refreshBtn.disabled = true;
    refreshBtn.classList.add('checking');
    refreshBtn.textContent = 'Checking...';

    try {
        // Update all stream statuses
        const result = await updateQuickStreamStatus();

        // Reload settings and update UI to show new statuses
        await loadSettingsAndUpdateUI();

    } catch (error) {
        console.error('Failed to refresh stream status:', error);
        showMessage('Failed to refresh stream statuses: ' + error, 'error');
    } finally {
        refreshBtn.disabled = false;
        refreshBtn.classList.remove('checking');
        refreshBtn.textContent = originalText;
    }
}

// Load settings and update UI
export async function loadSettingsAndUpdateUI() {
    try {
        const settings = await loadSettings();

        appState.selectedQuality = settings.default_quality || 'best';
        appState.quickStreams = settings.quick_streams || [];

        // Update quality selector
        elements.qualitySelect.value = appState.selectedQuality;

    } catch (error) {
        console.error('Failed to load settings:', error);
        throw error;
    }

    // Update UI after settings are loaded
    await updateUI();
}

// Handle keyboard shortcuts
export function handleKeydown(e) {
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

// Setup all event listeners
export function setupEventListeners() {
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