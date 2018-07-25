let pushServer = null;
let searchBox = null;

document.addEventListener("DOMContentLoaded", () => {
    // Setup push messages
    pushServer = new EventSource(document.body.dataset.pushEndpoint);
    pushServer.addEventListener("comment", handleCommentPush);
    pushServer.addEventListener("completion", handleTaskPush);
    pushServer.addEventListener("elaboration", handleExperimentPush);

    for(input of document.querySelectorAll(".task input")) {
        input.addEventListener("change", handleTaskChange);
    }

    for(select of document.querySelectorAll("select.elaboration")) {
        // Remember previous value for the reset logic on fetch failure
        select.dataset.prev_selected = select.selectedIndex;
        select.addEventListener("change", handleElaborationChange);
    }

    for(saveButton of document.querySelectorAll(".comment button.save")) {
        saveButton.addEventListener("click", handleCommentSave);
    }

    for(addDate of document.querySelectorAll(".comment button.date")) {
        addDate.addEventListener("click", handleCommentDate);
    }

    for(comment of document.querySelectorAll(".comment textarea")) {
        comment.addEventListener("input", (event) => {
            event.target.closest(".comment").classList.add("unsaved");
        });
    }

    for(let students of document.querySelectorAll("ul.students")) {
        students.addEventListener("click", handleStudentClick);
    }

    searchBox = new OverlaySearchBox(document.querySelector("#overlay .search"),
        searchStudents);

    document.querySelector("#add-group").addEventListener("click", onNewGroup);

    for(let editButton of document.querySelectorAll(".group h2 img")) {
        editButton.addEventListener("click", onGroupDeskChange);
    }

    document.querySelector("header .stats")
        .addEventListener("click", toggleCompactMode);

    restoreSessionState();
});

function promptInt(message) {
    let input = prompt(message);
    if(input == null) {
        return null;
    }

    let desk = parseInt(input, 10);
    if(isNaN(desk)) {
        toast("error", "Keine gültige Tischnummer.");
        return null;
    }

    return desk;
}

async function onNewGroup(event) {
    let desk = promptInt("Tischnumer der neuen Gruppe:");
    if(desk === null) {
        return;
    }

    let day = document.querySelector(".experiment").dataset.day;

    try {
        let url = "/api/group";

        let response = await myfetch(url, {
            method: "POST",
            headers: new Headers({"Content-Type": "application/json"}),
            body: JSON.stringify({
                desk: desk,
                day_id: parseInt(day, 10),
                comment: ""
            })
        });
        handleResponse(response);

        toast("reload", "Die neue Gruppe wurde hinzugefügt. Seite neuladen um sie anzuzeigen oder weitere Gruppen hinzufügen!")
    } catch(e) {
        toast("error", e);
    }
}

async function onGroupDeskChange(event) {
    // do not reload page (empty href)
    event.preventDefault();

    let desk = promptInt("Neue Tischnumer der Gruppe:");
    if(desk === null) {
        return;
    }

    let group = event.target.closest(".group").dataset.id;

    try {
        let url = "/api/group/" + group + "/desk";

        let response = await myfetch(url, {
            method: "PUT",
            headers: new Headers({"Content-Type": "application/json"}),
            body: JSON.stringify(desk)
        });
        handleResponse(response);

        toast("reload", "Die Tischnummer der Gruppe wurde geändert. Seite neuladen um sie anzuzeigen!")
    } catch(e) {
        toast("error", e);
    }
}

function toggleCompactMode() {
    let experiment = document.querySelector("#main > .experiment");
    let compactActive = experiment.classList.contains("compact");

    if(compactActive) {
        experiment.classList.remove("compact");
        sessionStorage.removeItem("compact");
    } else {
        experiment.classList.add("compact");
        sessionStorage.setItem("compact", "yes");
    }
}

function restoreSessionState() {
    if(sessionStorage.getItem("compact") === "yes") {
        document.querySelector("#main > .experiment").classList.add("compact");
    }
}
