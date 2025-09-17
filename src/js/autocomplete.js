// Autocomplete prevention functionality

// Disable autocomplete on all inputs with comprehensive protection
export function disableAutocomplete() {
    const inputs = document.querySelectorAll('input');
    inputs.forEach(input => {
        // Basic HTML attributes (first layer)
        input.setAttribute('autocomplete', 'off');
        input.setAttribute('autocorrect', 'off');
        input.setAttribute('autocapitalize', 'off');
        input.setAttribute('spellcheck', 'false');
        input.setAttribute('data-form-type', 'other');
        input.setAttribute('data-lpignore', 'true'); // LastPass ignore
        input.setAttribute('data-1p-ignore', 'true'); // 1Password ignore

        // Force form association to prevent autofill
        if (!input.form) {
            const form = document.createElement('form');
            form.style.display = 'none';
            form.setAttribute('autocomplete', 'off');
            document.body.appendChild(form);
            form.appendChild(input);
        } else {
            input.form.setAttribute('autocomplete', 'off');
        }

        // Event-based protection (second layer)
        const preventAutocomplete = (e) => {
            e.target.setAttribute('autocomplete', 'off');
            e.target.style.webkitAppearance = 'none';

            // Clear any existing autocomplete data
            if (e.target.value && e.target.dataset.originalValue !== e.target.value) {
                e.target.dataset.originalValue = e.target.value;
            }
        };

        input.addEventListener('focus', preventAutocomplete);
        input.addEventListener('input', preventAutocomplete);
        input.addEventListener('keydown', preventAutocomplete);

        // Blur event to ensure autocomplete stays disabled
        input.addEventListener('blur', (e) => {
            setTimeout(() => {
                e.target.setAttribute('autocomplete', 'off');
            }, 100);
        });

        // Aggressive mutation observer (third layer)
        const observer = new MutationObserver((mutations) => {
            mutations.forEach((mutation) => {
                if (mutation.type === 'attributes' &&
                    (mutation.attributeName === 'autocomplete' ||
                     mutation.attributeName === 'autocorrect' ||
                     mutation.attributeName === 'autocapitalize')) {
                    if (input.getAttribute(mutation.attributeName) !== 'off') {
                        input.setAttribute(mutation.attributeName, 'off');
                    }
                }
            });
        });

        observer.observe(input, {
            attributes: true,
            attributeFilter: ['autocomplete', 'autocorrect', 'autocapitalize']
        });

        // Store observer reference for cleanup if needed
        input._autocompleteObserver = observer;

        // CSS injection for additional protection (fourth layer)
        const styleId = 'autocomplete-disable-' + Math.random().toString(36).substr(2, 9);
        const style = document.createElement('style');
        style.id = styleId;
        style.textContent = `
            #${input.id || input.name || 'input-' + Math.random().toString(36).substr(2, 9)} {
                -webkit-autofill: none !important;
                -webkit-box-shadow: 0 0 0 1000px var(--surface) inset !important;
                -webkit-text-fill-color: var(--text) !important;
                background-clip: padding-box !important;
            }
            #${input.id || input.name || 'input-' + Math.random().toString(36).substr(2, 9)}::-webkit-autofill {
                display: none !important;
            }
        `;
        document.head.appendChild(style);
        input._autocompleteStyle = style;
    });
}