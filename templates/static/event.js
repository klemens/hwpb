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
            if(!comment.value.endsWith("\n")) {
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

    searchBox = new SearchBox(document.querySelector(".search"));

    for(let students of document.querySelectorAll("ul.students")) {
        students.addEventListener("click", onStudentClick);
    }

    document.querySelector("#add-group").addEventListener("click", onNewGroup);

    for(let editButton of document.querySelectorAll(".group h2 img")) {
        editButton.addEventListener("click", onGroupDeskChange);
    }
});

class SearchBox {
    constructor(element) {
        this.box = element;
        this.loadTimer = null;
        this.successCallback = null;

        let input = element.querySelector("input");
        input.addEventListener("keydown", this.onInputKey.bind(this));
        input.addEventListener("input", this.onInputChange.bind(this));

        let overlay = element.closest("#overlay");
        overlay.addEventListener("click", event => {
            if(event.target === overlay) {
                this.deactivate();
            }
        });
    }

    activate(successCallback) {
        this.successCallback = successCallback;
        this.box.closest("#overlay").classList.add("active");
        this.box.querySelector("input").focus();
    }

    deactivate() {
        this.box.closest("#overlay").classList.remove("active");
        this.box.querySelector("input").value = "";
        this.clearStudents();
    }

    clearStudents() {
        let studentList = this.box.querySelector("ul");
        while(studentList.firstChild) {
            studentList.removeChild(studentList.firstChild);
        }
    }

    insertStudents(students) {
        let studentList = this.box.querySelector("ul");

        for(let student of students) {
            let node = document.createElement("li");
            node.textContent = student.name;
            node.addEventListener("click", this.onStudentSelected.bind(this));
            node.dataset.id = student.id;
            node.dataset.name = student.name;
            studentList.appendChild(node);
        }

        // Select the first student if any
        let firstStudent = studentList.querySelector("li");
        if(firstStudent) {
            firstStudent.classList.add("selected");
        }
    }

    onInputKey(event) {
        let selected = this.box.querySelector("li.selected");

        // Abort if there are no students in the list (one is always active)
        if(!selected) {
            return;
        }

        switch(event.keyCode) {
            case 13: // return
                this.onStudentSelected(selected);
                break;

            case 38: // up
                if(selected.previousElementSibling !== null) {
                    selected.classList.remove("selected");
                    selected.previousElementSibling.classList.add("selected");
                }
                break;

            case 40: // down
                if(selected.nextElementSibling !== null) {
                    selected.classList.remove("selected");
                    selected.nextElementSibling.classList.add("selected");
                }
                break;
        }
    }

    onInputChange() {
        if(this.loadTimer !== null) {
            clearTimeout(this.loadTimer);
        }

        let terms = this.box.querySelector("input").value;
        terms = terms.split(" ").filter(x => x);

        // Do not list all students when no terms are given
        if(terms.length === 0) {
            this.clearStudents();
            return;
        }

        this.loadTimer = setTimeout(async () => {
            try {
                let response = await myfetch("/api/student/search", {
                    method: "POST",
                    headers: new Headers({"Content-Type": "application/json"}),
                    body: JSON.stringify(terms)
                });
                if(!response.ok) {
                    throw "API error";
                }

                let students = await response.json();

                this.clearStudents();
                this.insertStudents(students);
            } catch(e) {
                toast("error", e);
            }
        }, 250);
    }

    onStudentSelected(selected) {
        // Support both events and elements
        if(selected.target) {
            selected = selected.target;
        }

        if(this.successCallback !== null) {
            let data = selected.closest("li").dataset;
            this.successCallback(data.id, data.name);
        }
    }
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
