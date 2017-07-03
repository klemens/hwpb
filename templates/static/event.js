let searchBox = null;

document.addEventListener("DOMContentLoaded", () => {
    for (input of document.querySelectorAll(".task input")) {
        input.addEventListener("change", async (event) => {
            let checked = event.target.checked;

            let group = event.target.closest(".group").dataset.id;
            let task = event.target.closest(".task").dataset.id;

            try {
                let url = "/api/group/" + group + "/completed/" + task;
                let options = {
                    method: checked ? "PUT" : "DELETE"
                };

                let response = await myfetch(url, options);
                if(!response.ok) {
                    throw "API error";
                }
            } catch(e) {
                toast("error", e);
                event.target.checked = !checked;
            }
        });
    }

    for(select of document.querySelectorAll("select.elaboration")) {
        // Remember previous value for the reset logic on fetch failure
        select.dataset.prev_selected = select.selectedIndex;

        select.addEventListener("change", async (event) => {
            let data = event.target.selectedOptions[0].dataset;

            let group = event.target.closest(".group").dataset.id;
            let experiment = event.target.closest(".experiment").dataset.id;

            try {
                let url = "/api/group/" + group + "/elaboration/" + encodeURI(experiment);

                let response = null;
                if(data.accepted !== undefined) {
                    response = await myfetch(url, {
                        method: "PUT",
                        headers: new Headers({"Content-Type": "application/json"}),
                        body: JSON.stringify({
                            rework_required: data.rework == "1",
                            accepted: data.accepted == "1"
                        })
                    });
                } else {
                    response = await myfetch(url, {
                        method: "DELETE"
                    });
                }
                if(!response.ok) {
                    throw "API error";
                }

                event.target.dataset.prev_selected = event.target.selectedIndex;
            } catch(e) {
                toast("error", e);
                event.target.selectedIndex = event.target.dataset.prev_selected;
            }
        });
    }

    for(saveButton of document.querySelectorAll(".comment button.save")) {
        saveButton.addEventListener("click", async (event) => {
            let group = event.target.closest(".group").dataset.id;
            let comment = event.target.closest(".comment").querySelector("textarea").value;

            try {
                let url = "/api/group/" + group + "/comment";

                let response = await myfetch(url, {
                    method: "PUT",
                    headers: new Headers({"Content-Type": "application/json"}),
                    body: JSON.stringify(comment)
                });
                if(!response.ok) {
                    throw "API error";
                }

                event.target.closest(".comment").classList.remove("unsaved");
            } catch(e) {
                toast("error", e);
            }
        });
    }

    for(addDate of document.querySelectorAll(".comment button.date")) {
        addDate.addEventListener("click", (event) => {
            let comment = event.target.closest(".comment").querySelector("textarea");

            let value = "";
            if(!comment.value.endsWith("\n") && comment.value !== "") {
                value += "\n";
            }
            value += new Date().toISOString().substr(0, 10);
            value += ": ";

            comment.focus();
            comment.value += value;

            event.target.closest(".comment").classList.add("unsaved");
        });
    }

    for(comment of document.querySelectorAll(".comment textarea")) {
        comment.addEventListener("input", (event) => {
            event.target.closest(".comment").classList.add("unsaved");
        });
    }

    searchBox = new OverlaySearchBox(document.querySelector(".search"), searchStudents);

    for(let students of document.querySelectorAll("ul.students")) {
        students.addEventListener("click", onStudentClick);
    }

    document.querySelector("#add-group").addEventListener("click", onNewGroup);

    for(let editButton of document.querySelectorAll(".group h2 img")) {
        editButton.addEventListener("click", onGroupDeskChange);
    }
});

async function searchStudents(terms) {
    let response = await myfetch("/api/student/search", {
        method: "POST",
        headers: new Headers({"Content-Type": "application/json"}),
        body: JSON.stringify(terms)
    });
    if(!response.ok) {
        throw "API error";
    }

    return response.json();
}

// Search/add students and delete them after confirmation
async function onStudentClick(event) {
    let target = event.target;
    let group = target.closest(".group").dataset.id;

    if(!(target instanceof HTMLLIElement)) {
        return;
    }

    if(target.classList.contains("add")) {
        searchBox.activate(async (studentId, studentName) => {
            let node = document.createElement("li");
            node.textContent = studentName;
            node.dataset.id = studentId;
            event.target.closest("ul").appendChild(node);

            try {
                let url = "/api/group/" + group + "/student/" + studentId;

                let response = await myfetch(url, {
                    method: "PUT"
                });
                if(!response.ok) {
                    throw "API error";
                }

                searchBox.deactivate();
            } catch(e) {
                toast("error", e);
                event.target.closest("ul").removeChild(node);
            }
        });
    } else {
        let studentId = target.dataset.id;
        let studentName = target.textContent;

        if(!confirm(studentName + " wirklich aus der Gruppe entfernen?")) {
            return;
        }

        let parent = target.parentNode;
        parent.removeChild(target);

        try {
            let url = "/api/group/" + group + "/student/" + studentId;

            let response = await myfetch(url, {
                method: "DELETE"
            });
            if(!response.ok) {
                throw "API error";
            }

            searchBox.deactivate();
        } catch(e) {
            toast("error", e);
            parent.appendChild(target);
        }
    }
}

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
                day_id: day,
                comment: ""
            })
        });
        if(!response.ok) {
            throw "API error";
        }

        toast("info", "Die neue Gruppe wurde hinzugefügt. Seite neuladen um sie anzuzeigen!")
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
        if(!response.ok) {
            throw "API error";
        }

        toast("info", "Die Tischnummer der Gruppe wurde geändert. Seite neuladen um sie anzuzeigen!")
    } catch(e) {
        toast("error", e);
    }
}
