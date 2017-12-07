document.addEventListener("DOMContentLoaded", () => {
    for(let day of document.querySelectorAll(".day h2")) {
        day.addEventListener("click", onDeleteDay);
    }

    document.querySelector("#add-day")
        .addEventListener("click", onNewDay);
});


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
        if(!response.ok) {
            throw "API error";
        }

        day.parentNode.removeChild(day);
    } catch(e) {
        toast("error", e);
    }
}

async function onNewDay() {
    let day = prompt("Name des neuen Versuchstages (z. B. Di-A):");
    if(day === null) {
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
        if(!response.ok) {
            throw "API error";
        }

        // reload to avoid rendering complex structures on the client
        location.reload(true);
    } catch(e) {
        toast("error", e);
    }
}
