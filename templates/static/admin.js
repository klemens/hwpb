document.addEventListener("DOMContentLoaded", () => {
    let yearSelect = document.querySelector("header select");
    yearSelect.addEventListener("change", () => {
        let site = document.body.dataset.site;
        document.location = "/admin/" + yearSelect.value + "/" + site;
    });
});
