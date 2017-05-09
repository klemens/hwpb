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
