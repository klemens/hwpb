document.addEventListener("DOMContentLoaded", () => {
    let overlay = document.querySelector("#overlay");

    overlay.addEventListener("click", hideOverlay);

    document.querySelector("#import-students")
        .addEventListener("click", showOverlay);

    document.querySelector("#students-csv")
        .addEventListener("change", importStudents);

    document.querySelector("#add-student")
        .addEventListener("submit", onNewStudent);

    for(let removeButton of document.querySelectorAll("table .button.remove")) {
        removeButton.addEventListener("click", onDeleteStudent)
    }

    for(let instructedCheckbox of document.querySelectorAll("td.instructed input")) {
        instructedCheckbox.addEventListener("change", onChangeInstructed);
    }
});

function hideOverlay(event) {
    let overlay = document.querySelector("#overlay");

    if(event && event.target !== overlay) {
        return;
    }

    overlay.classList.remove("active");
}

function showOverlay() {
    document.querySelector("#students-csv").value = "";
    document.querySelector("#overlay").classList.add("active");
}

async function importStudents(event) {
    let files = event.target.files;
    if(files.length === 0) {
        return;
    }

    let year = parseInt(document.body.dataset.year);

    try {
        let url = "/api/students/" + year;

        let response = await myfetch(url, {
            method: "POST",
            headers: new Headers({"Content-Type": "text/csv"}),
            body: files[0]
        });
        handleResponse(response);

        // reload to avoid rendering on the client
        location.reload(true);
    } catch(e) {
        toast("error", e);
        hideOverlay();
    }
}

async function onNewStudent(event) {
    event.preventDefault();

    let matrikel = document.querySelector("#add-student input[name='matrikel']").value;
    let name = document.querySelector("#add-student input[name='name']").value;
    let username = document.querySelector("#add-student input[name='username']").value;
    let year = parseInt(document.body.dataset.year);

    matrikel = matrikel.trim();
    name = name.trim();
    if(matrikel.length == 0 || name.length == 0) {
        toast("error", "Ungültige Eingabe");
        return;
    }

    username = username.trim();
    if(username.length == 0) {
        username = null;
    }

    try {
        let url = "/api/student";

        let response = await myfetch(url, {
            method: "POST",
            headers: new Headers({"Content-Type": "application/json"}),
            body: JSON.stringify({
                matrikel: matrikel,
                name: name,
                year: year,
                username: username
            })
        });
        handleResponse(response);

        // reload to avoid rendering on the client
        location.reload(true);
    } catch(e) {
        toast("error", e);
    }
}

async function onDeleteStudent(event) {
    let targetRow = event.target.closest("tr");

    let id = targetRow.dataset.id;
    let name = targetRow.querySelector("td:nth-of-type(2)").textContent;

    if(!confirm(name + " wirklich löschen?")) {
        return;
    }

    try {
        let url = "/api/student/" + id;

        let response = await myfetch(url, {
            method: "DELETE"
        });
        handleResponse(response);

        targetRow.parentNode.removeChild(targetRow);
    } catch(e) {
        toast("error", e);
    }
}

async function onChangeInstructed(event) {
    let target = event.target;

    let id = target.closest("tr").dataset.id;
    let instructed = target.checked;

    try {
        let url = "/api/student/" + id + "/instructed";

        let response = await myfetch(url, {
            method: "PUT",
            headers: new Headers({"Content-Type": "application/json"}),
            body: JSON.stringify(instructed)
        });
        handleResponse(response);
    } catch(e) {
        toast("error", e);
        target.checked = !instructed;
    }

}
