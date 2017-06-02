/// fetch with support for deadlines (timeout)
///
/// *Warning*: This function "leaks" the connection when a timeout occures.
/// See https://github.com/whatwg/fetch/issues/179 for details.
function myfetch(input, options) {
    return new Promise((resolve, reject) => {
        if(options.deadline) {
            setTimeout(() => {
                reject(new Error("Timeout: deadline reached"))
            }, options.deadline);
        }

        // always send cookies
        options.credentials = 'same-origin';

        fetch(input, options).then(resolve, reject);
    });
}

function toast(type, message) {
    let toast = document.createElement("div");
    toast.classList.add("toast");
    toast.classList.add(type);
    toast.textContent = "Fehler: " + message;

    document.body.appendChild(toast);
    setTimeout(() => {
        document.body.removeChild(toast);
    }, 7500);
}
