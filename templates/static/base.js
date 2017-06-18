/// fetch with support for deadlines (timeout)
///
/// *Warning*: This function "leaks" the connection when a timeout occures.
/// See https://github.com/whatwg/fetch/issues/179 for details.
function myfetch(input, options) {
    return new Promise((resolve, reject) => {
        // default timeout is 4 seconds
        let timeout = options.deadline || 4000;

        setTimeout(() => {
            reject(new Error("Timeout: deadline reached"))
        }, timeout);

        // always send cookies
        options.credentials = 'same-origin';

        fetch(input, options).then(resolve, reject);
    });
}

function toast(type, message) {
    let prefix = "";
    if(type === "error") {
        prefix = "Fehler: ";
    } else if(type === "info") {
        prefix = "Info: ";
    }

    let toast = document.createElement("div");
    toast.classList.add("toast");
    toast.classList.add(type);
    toast.textContent = prefix + message;

    document.body.appendChild(toast);
    setTimeout(() => {
        document.body.removeChild(toast);
    }, 7500);
}
