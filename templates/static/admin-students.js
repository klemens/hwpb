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
        if(!response.ok) {
            throw "API error";
        }

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
    let year = parseInt(document.body.dataset.year);

    matrikel = matrikel.trim();
    name = name.trim();
    if(matrikel.length == 0 || name.length == 0) {
        toast("error", "Ungültige Eingabe");
        return;
    }

    try {
        let url = "/api/student";

        let response = await myfetch(url, {
            method: "POST",
            headers: new Headers({"Content-Type": "application/json"}),
            body: JSON.stringify({
                matrikel: matrikel,
                name: name,
                year: year
            })
        });
        if(!response.ok) {
            throw "API error";
        }

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
        if(!response.ok) {
            throw "API error";
        }

        targetRow.parentNode.removeChild(targetRow);
    } catch(e) {
        toast("error", e);
    }
}
