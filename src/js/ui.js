// UI management and updates
import { appState, elements } from './state.js';
import { showMessage } from './utils.js';

// Update main action button
export function updateMainButton() {
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
export function updateStatusIndicator() {
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
            // This would need to be imported from api.js
            elements.invoke('extract_channel_name', { url: appState.currentUrl })
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
export function updateQuickStreams() {
    elements.quickStreamSlots.forEach((slot, index) => {
        const stream = appState.quickStreams[index];

        // Clear existing content
        slot.innerHTML = '';

        if (stream) {
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

// Update UI based on current state
export async function updateUI() {
    updateMainButton();
    updateStatusIndicator();
    updateQuickStreams();

    // Check current stream status
    try {
        // This would need to be imported from api.js
        const currentStream = await elements.invoke('get_current_stream');
        if (currentStream) {
            appState.isStreaming = true;
            appState.currentUrl = currentStream;
        }
    } catch (error) {
        console.error('Failed to get current stream:', error);
    }
}

// Set starting state
export function setStartingState(starting) {
    appState.isStartingStream = starting;
    updateMainButton();
    updateStatusIndicator();
}

// Show add stream modal
export function showAddStreamModal(index) {
    elements.addStreamModal.classList.add('show');
    elements.addStreamModal.dataset.index = index;
    elements.streamName.value = '';
    elements.streamUrl.value = '';
    elements.streamName.focus();
}

// Hide add stream modal
export function hideAddStreamModal() {
    elements.addStreamModal.classList.remove('show');
}

// Show context menu
export function showContextMenu(x, y, index) {
    elements.contextMenu.style.left = x + 'px';
    elements.contextMenu.style.top = y + 'px';
    elements.contextMenu.style.display = 'block';
    elements.contextMenu.dataset.index = index;
}

// Hide context menu
export function hideContextMenu() {
    elements.contextMenu.style.display = 'none';
}