// Utility functions

// Shared function to get user-friendly error messages
export function getErrorMessage(error) {
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

// Show message to user
export function showMessage(message, type = 'info') {
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

export function hideMessage() {
    const messageEl = elements.messageDisplay;
    messageEl.classList.remove('show');
    setTimeout(() => {
        messageEl.className = 'message-display';
        messageEl.innerHTML = '';
    }, 300);
}