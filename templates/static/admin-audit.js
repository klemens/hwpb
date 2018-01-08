document.addEventListener("DOMContentLoaded", () => {
    let form = document.querySelector("form.filter");

    for(filter of document.querySelectorAll("form.filter select")) {
        filter.addEventListener("change", form.submit.bind(form));
    }

    document.querySelector("form.filter p.limit a")
        .addEventListener("click", onRemoveLimit);
});

function onRemoveLimit(event) {
    event.preventDefault();

    let form = event.target.closest("form");

    form.querySelector("input[name='limit'").disabled = true;
    form.submit();
}
