document.addEventListener("DOMContentLoaded", () => {
    document.querySelector("header select")
        .addEventListener("change", onYearChange);

    let closeYear = document.querySelector("#close-year");
    if(closeYear !== null) {
        closeYear.addEventListener("click", onCloseYear);
    }
});

async function onYearChange(event) {
    let target = event.target;

    if(target.value === "new-year") {
        let year = prompt("Neues Jahr:");
        if(year === null) {
            // reset select to currently loaded year
            target.value = document.body.dataset.year;
            return;
        }

        year = parseInt(year);
        if(isNaN(year)) {
            toast("error", "Ungültiges Jahr");
            // reset select to currently loaded year
            target.value = document.body.dataset.year;
            return;
        }

        try {
            let url = "/api/year/" + year;

            let response = await myfetch(url, {
                method: "PUT"
            });
            handleResponse(response);

            location = "/admin/" + year;
        } catch(e) {
            toast("error", e);
        }
    } else {
        let site = document.body.dataset.site;
        location = "/admin/" + target.value + "/" + site;
    }
}

async function onCloseYear() {
    let year = document.body.dataset.year;

    let confirmMessage =
        "Möchten Sie das Jahr " + year + " wirklich abschließen und damit " +
        "weitere Änderungen unterbinden?\n\n" +
        "Das Abschließen eines Jahres ist endgültig und sollte erst einige " +
        "Zeit nach Abschluss des Praktikums erfolgen (z. B. nach Übermittlung " +
        "der zugelassenen Studenten an das Prüfungsamt).";

    if(!confirm(confirmMessage)) {
        return;
    }

    try {
        let url = "/api/year/" + year + "/closed";

        let response = await myfetch(url, {
            method: "PUT"
        });
        handleResponse(response);

        location.reload();
    } catch(e) {
        toast("error", e);
    }
}
