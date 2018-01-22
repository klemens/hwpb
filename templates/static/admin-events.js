document.addEventListener("DOMContentLoaded", () => {
    for(let input of document.querySelectorAll(".experiment input.date")) {
        input.dataset.prev_value = input.value;
        input.addEventListener("input", onInputDate);
        input.addEventListener("change", onChangeDate);
    }

    for(let day of document.querySelectorAll(".day h2")) {
        day.addEventListener("click", onDeleteDay);
    }

    document.querySelector("#add-day")
        .addEventListener("click", onNewDay);
});

function onInputDate(event) {
    let target = event.target;

    // the change event is not fired when clearing date in recent firefox
    // without setting it beforehand, so always save when the date was cleared;
    // this leads to multiple calls to onChangeDate on browsers that behave
    // correctly, which is not a huge problem, because the api is idempotent
    if(target.value === "") {
        onChangeDate(event);
    }

    if(target.dataset.prev_value === target.value) {
        target.classList.remove("dirty");
    } else {
        target.classList.add("dirty");
    }
}

async function onChangeDate(event) {
    let target = event.target;
    let date = target.value;

    let experiment = target.closest(".experiment").dataset.id;
    let day = target.closest(".day").dataset.id;

    try {
        let url = "/api/experiment/" + experiment + "/day/" + day + "/event";

        let request = {
            headers: new Headers({"Content-Type": "application/json"})
        };
        if(date) {
            request.method = "PUT";
            request.body = JSON.stringify(date);
        } else {
            request.method = "DELETE";
        }

        let response = await myfetch(url, request);
        handleResponse(response);

        target.dataset.prev_value = target.value;
    } catch(e) {
        toast("error", e);
        target.value = target.dataset.prev_value;
    }

    target.classList.remove("dirty");
}

async function onDeleteDay(event) {
    let target = event.target;

    let day = target.closest(".day");
    let dayId = day.dataset.id;
    let dayName = day.querySelector("h2").textContent;

    if(!confirm(dayName + " wirklich löschen?")) {
        return;
    }

    try {
        let url = "/api/day/" + dayId;

        let response = await myfetch(url, {
            method: "DELETE"
        });
        handleResponse(response);

        day.parentNode.removeChild(day);
    } catch(e) {
        toast("error", e);
    }
}

async function onNewDay() {
    let day = prompt("Name des neuen Versuchstages (z. B. Di-A):");
    if(day === null || day.trim() === "") {
        return;
    }

    let year = parseInt(document.body.dataset.year);

    try {
        let url = "/api/day";

        let response = await myfetch(url, {
            method: "POST",
            headers: new Headers({"Content-Type": "application/json"}),
            body: JSON.stringify({
                name: day.trim(),
                year: year
            })
        });
        handleResponse(response);

        // reload to avoid rendering complex structures on the client
        location.reload(true);
    } catch(e) {
        toast("error", e);
    }
}
