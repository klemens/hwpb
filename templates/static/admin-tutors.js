document.addEventListener("DOMContentLoaded", () => {
    document.querySelector("#add-tutor")
        .addEventListener("submit", onNewTutor);
    document.querySelector("#add-ip-whitelist")
        .addEventListener("submit", onNewIpWhitelistEntry);

    for(let removeButton of document.querySelectorAll("#add-tutor .button.remove")) {
        removeButton.addEventListener("click", onDeleteTutor);
    }
    for(let removeButton of document.querySelectorAll("#add-ip-whitelist .button.remove")) {
        removeButton.addEventListener("click", onDeleteIpWhitelistEntry);
    }

    for(let adminCheckbox of document.querySelectorAll("#add-tutor input.admin")) {
        adminCheckbox.addEventListener("change", onChangeAdmin);
    }
});

async function onNewTutor(event) {
    event.preventDefault();

    let username = document.querySelector("#add-tutor input[name='username']").value;
    let year = parseInt(document.body.dataset.year);

    username = username.trim();
    if(username.length == 0) {
        toast("error", "Ungültige Eingabe");
        return;
    }

    try {
        let url = "/api/tutor";

        let response = await myfetch(url, {
            method: "POST",
            headers: new Headers({"Content-Type": "application/json"}),
            body: JSON.stringify({
                username: username,
                year: year,
                is_admin: false
            })
        });
        handleResponse(response);

        // reload to avoid rendering on the client
        location.reload(true);
    } catch(e) {
        toast("error", e);
    }
}

async function onNewIpWhitelistEntry(event) {
    event.preventDefault();

    let ipnet = document.querySelector("#add-ip-whitelist input[name='ipnet']").value.trim();
    if(ipnet.length == 0) {
        toast("error", "Ungültige Eingabe");
        return;
    }
    let year = parseInt(document.body.dataset.year);

    try {
        let url = "/api/ip-whitelist";

        let response = await myfetch(url, {
            method: "POST",
            headers: new Headers({"Content-Type": "application/json"}),
            body: JSON.stringify({
                ipnet: ipnet,
                year: year,
            })
        });
        handleResponse(response, {
            500: "Serverfehler (eventuell falsche IP-Syntax?)"
        });

        // reload to avoid rendering on the client
        location.reload(true);
    } catch(e) {
        toast("error", e);
    }
}

async function onDeleteTutor(event) {
    let targetRow = event.target.closest("tr");

    let id = targetRow.dataset.id;
    let name = targetRow.querySelector("td:nth-of-type(1)").textContent;

    if(!confirm(name + " wirklich löschen?")) {
        return;
    }

    try {
        let url = "/api/tutor/" + id;

        let response = await myfetch(url, {
            method: "DELETE"
        });
        handleResponse(response);

        targetRow.parentNode.removeChild(targetRow);
    } catch(e) {
        toast("error", e);
    }
}

async function onDeleteIpWhitelistEntry(event) {
    let targetRow = event.target.closest("tr");

    let id = targetRow.dataset.id;
    let name = targetRow.querySelector("td:nth-of-type(1)").textContent;

    if(!confirm(name + " wirklich löschen?")) {
        return;
    }

    try {
        let url = "/api/ip-whitelist/" + id;

        let response = await myfetch(url, {
            method: "DELETE"
        });
        handleResponse(response);

        targetRow.parentNode.removeChild(targetRow);
    } catch(e) {
        toast("error", e);
    }
}

async function onChangeAdmin(event) {
    let target = event.target;

    let id = target.closest("tr").dataset.id;
    let is_admin = target.checked;

    try {
        let url = "/api/tutor/" + id + "/is_admin";

        let response = await myfetch(url, {
            method: "PUT",
            headers: new Headers({"Content-Type": "application/json"}),
            body: JSON.stringify(is_admin)
        });
        handleResponse(response);
    } catch(e) {
        toast("error", e);
        target.checked = !is_admin;
    }

}
