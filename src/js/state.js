// Application state management
const appState = {
    isStreaming: false,
    currentUrl: '',
    selectedQuality: 'best',
    quickStreams: [],
    isStartingStream: false
};

// DOM elements cache
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

export { appState, elements };