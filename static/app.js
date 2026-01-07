/**
 * HEIC to JPG Converter - Frontend Logic
 */

// DOM elements
const dropZone = document.getElementById('dropZone');
const fileInput = document.getElementById('fileInput');
const fileList = document.getElementById('fileList');
const qualityInputs = document.querySelectorAll('input[name="quality"]');
const qualityValue = document.getElementById('qualityValue');
const convertBtn = document.getElementById('convertBtn');
const resultList = document.getElementById('results');
const fileListActions = document.getElementById('fileListActions');
const clearAllBtn = document.getElementById('clearAllBtn');

// State
let files = [];
let isConverting = false;

// Initialize
function init() {
    setupDragDrop();
    setupFileInput();
    setupQualitySlider();
    setupQualitySlider();
    setupConvertButton();
    setupClearAllButton();
}

// Drag and drop handling
function setupDragDrop() {
    dropZone.addEventListener('click', () => fileInput.click());

    dropZone.addEventListener('dragover', (e) => {
        e.preventDefault();
        dropZone.classList.add('dragover');
    });

    dropZone.addEventListener('dragleave', (e) => {
        e.preventDefault();
        dropZone.classList.remove('dragover');
    });

    dropZone.addEventListener('drop', (e) => {
        e.preventDefault();
        dropZone.classList.remove('dragover');

        const droppedFiles = Array.from(e.dataTransfer.files).filter(isHeicFile);
        addFiles(droppedFiles);
    });
}

// File input handling
function setupFileInput() {
    fileInput.addEventListener('change', (e) => {
        const selectedFiles = Array.from(e.target.files).filter(isHeicFile);
        addFiles(selectedFiles);
        fileInput.value = ''; // Reset for re-selection
    });
}

// Check if file is HEIC
function isHeicFile(file) {
    const name = file.name.toLowerCase();
    return name.endsWith('.heic') || name.endsWith('.heif');
}

// Add files to the list
function addFiles(newFiles) {
    for (const file of newFiles) {
        // Avoid duplicates
        if (!files.some(f => f.name === file.name && f.size === file.size)) {
            files.push(file);
        }
    }
    renderFileList();
    updateConvertButton();
}

// Remove file from list
function removeFile(index) {
    files.splice(index, 1);
    renderFileList();
    updateConvertButton();
}

// Format file size
function formatSize(bytes) {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
}

// Render file list
function renderFileList() {
    fileList.innerHTML = files.map((file, index) => `
        <div class="file-item" data-index="${index}">
            <div class="file-icon">📷</div>
            <div class="file-info">
                <div class="file-name">${escapeHtml(file.name)}</div>
                <div class="file-size">${formatSize(file.size)}</div>
            </div>
            <div class="file-status pending" id="status-${index}">Ready</div>
            <button class="file-remove" onclick="removeFile(${index})" title="Remove">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
            </button>
        </div>
        </div>
    `).join('');

    // Toggle actions visibility
    if (fileListActions) {
        fileListActions.style.display = files.length > 0 ? 'flex' : 'none';
    }
}

// Escape HTML for safety
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Quality presets
function setupQualitySlider() {
    qualityInputs.forEach(input => {
        input.addEventListener('change', (e) => {
            const val = e.target.value;
            let text = 'Recommended';
            if (val === '95') text = 'Best Quality';
            if (val === '60') text = 'Smallest Size';

            qualityValue.textContent = `${text} (${val}%)`;
        });
    });
}

// Convert button
function setupConvertButton() {
    convertBtn.addEventListener('click', convertAll);
}

function updateConvertButton() {
    convertBtn.disabled = files.length === 0 || isConverting;
    convertBtn.querySelector('.btn-text').textContent =
        files.length > 1 ? `Convert All (${files.length})` : 'Convert';
}

// Clear all button
function setupClearAllButton() {
    if (clearAllBtn) {
        clearAllBtn.addEventListener('click', clearAll);
    }
}

function clearAll() {
    if (isConverting) return;
    files = [];
    renderFileList();
    updateConvertButton();
}

// Convert all files
async function convertAll() {
    if (files.length === 0 || isConverting) return;

    isConverting = true;
    convertBtn.classList.add('loading');
    updateConvertButton();
    resultList.innerHTML = '';

    const selected = document.querySelector('input[name="quality"]:checked');
    const quality = parseInt(selected ? selected.value : '80');
    const filesToConvert = [...files];

    for (let i = 0; i < filesToConvert.length; i++) {
        const file = filesToConvert[i];
        updateFileStatus(i, 'converting', '<div class="spinner"></div> Converting...');

        try {
            const result = await convertFile(file, quality);
            updateFileStatus(i, 'done', '✓ Done');
            addResult(file.name, result);
        } catch (error) {
            updateFileStatus(i, 'error', '✗ ' + error.message);
            console.error('Conversion error:', error);
        }
    }

    isConverting = false;
    convertBtn.classList.remove('loading');
    updateConvertButton();
}

// Update file status in the list
function updateFileStatus(index, status, text) {
    const statusEl = document.getElementById(`status-${index}`);
    if (statusEl) {
        statusEl.className = `file-status ${status}`;
        statusEl.innerHTML = text;
    }
}

// Convert a single file
async function convertFile(file, quality) {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('quality', quality.toString());

    const response = await fetch('/api/convert', {
        method: 'POST',
        body: formData,
    });

    if (!response.ok) {
        const error = await response.json().catch(() => ({ error: 'Conversion failed' }));
        throw new Error(error.error || 'Conversion failed');
    }

    const blob = await response.blob();

    // Get filename from Content-Disposition header or generate one
    const contentDisposition = response.headers.get('Content-Disposition');
    let filename = file.name.replace(/\.heic$/i, '.jpg');
    if (contentDisposition) {
        const match = contentDisposition.match(/filename="?([^"]+)"?/);
        if (match) filename = match[1];
    }

    return { blob, filename };
}

// Add result to the results section
function addResult(originalName, { blob, filename }) {
    const url = URL.createObjectURL(blob);
    const size = formatSize(blob.size);

    const resultHtml = `
        <div class="result-item">
            <img class="result-preview" src="${url}" alt="Converted image">
            <div class="result-info">
                <div class="result-name">${escapeHtml(filename)}</div>
                <div class="result-meta">
                    ${size}
                </div>
            </div>
            <a href="${url}" download="${escapeHtml(filename)}" class="download-btn">
                Download
            </a>
        </div>
    `;

    results.insertAdjacentHTML('beforeend', resultHtml);

    // Auto-download
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
}

// Start the app
init();
