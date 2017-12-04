document.addEventListener("DOMContentLoaded", () => {
    let form = document.querySelector("form.filter");

    for(filter of document.querySelectorAll("form.filter select")) {
        filter.addEventListener("change", form.submit.bind(form));
    }
});
