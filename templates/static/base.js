/// fetch with support for deadlines (timeout)
///
/// *Warning*: This function "leaks" the connection when a timeout occures.
/// See https://github.com/whatwg/fetch/issues/179 for details.
function myfetch(input, options) {
    return new Promise((resolve, reject) => {
        // default timeout is 4 seconds
        let timeout = options.deadline || 4000;

        setTimeout(() => {
            reject(new Error("Timeout: deadline reached"))
        }, timeout);

        // always send cookies
        options.credentials = 'same-origin';

        fetch(input, options).then(resolve, reject);
    });
}

function toast(type, message) {
    let prefix = "";
    if(type === "error") {
        prefix = "Fehler: ";
    } else if(type === "info") {
        prefix = "Info: ";
    }

    let toast = document.createElement("div");
    toast.classList.add("toast");
    toast.classList.add(type);
    toast.textContent = prefix + message;

    document.body.appendChild(toast);
    setTimeout(() => {
        document.body.removeChild(toast);
    }, 7500);
}


function handleResponse(response, customErrors) {
    if(response.ok) {
        return;
    }

    const errors = {
        422: "Die Änderung konnte aufgrund von bestehenden Abhängigkeiten nicht durchgeführt werden.",
        423: "Die Änderung konnte nicht durchgeführt werden, da die Daten schreibgeschützt sind.",
        500: "Unbekannter Serverfehler"
    };

    let status = response.status;
    if(customErrors && customErrors[status]) {
        throw customErrors[status];
    } else if(errors[status]) {
        throw errors[status];
    } else {
        throw "Unbekannter Fehler: " + status;
    }
}

class SearchBox {
    constructor(element, searchCallback, successCallback) {
        this.box = element;
        this.loadTimer = null;
        this.searchCallback = searchCallback;
        this.successCallback = successCallback;

        let input = element.querySelector("input");
        input.addEventListener("keydown", this.onInputKey.bind(this));
        input.addEventListener("input", this.onInputChange.bind(this));
    }

    clear() {
        let list = this.box.querySelector("ul");
        while(list.firstChild) {
            list.removeChild(list.firstChild);
        }
        this.box.classList.remove("active");
    }

    insert(elements) {
        let list = this.box.querySelector("ul");

        for(let element of elements) {
            let node = document.createElement("li");
            node.addEventListener("click", this.onSelected.bind(this));

            if(element.href) {
                let link = document.createElement("a");
                link.href = element.href;
                link.textContent = element.text;
                node.appendChild(link);
            } else {
                node.textContent = element.text;
            }

            if(element.data) {
                for(const key of Object.keys(element.data)) {
                    node.dataset[key] = element.data[key];
                }
            }

            list.appendChild(node);
        }

        // Select the first student if any
        let firstElement = list.querySelector("li");
        if(firstElement) {
            firstElement.classList.add("selected");
            this.box.classList.add("active");
        }
    }

    onInputKey(event) {
        let selected = this.box.querySelector("li.selected");

        // Abort if there are no elements in the list (one is always active)
        if(!selected) {
            return;
        }

        switch(event.keyCode) {
            case 13: // return
                this.onSelected(selected);
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

        // Do not execute search when no terms are given
        if(terms.length === 0) {
            this.clear();
            return;
        }

        this.loadTimer = setTimeout(async () => {
            try {
                let elements = await this.searchCallback(terms);

                this.clear();
                this.insert(elements);
            } catch(e) {
                toast("error", e);
            }
        }, 250);
    }

    onSelected(selected) {
        // Support both events and elements
        if(selected.target) {
            selected = selected.target;
        }

        if(this.successCallback !== null) {
            let data = selected.closest("li").dataset;
            this.successCallback(data);
        }

        let link = selected.querySelector("a");
        if(link) {
            link.click();
        }
    }
}

class OverlaySearchBox extends SearchBox {
    constructor(element, searchCallback) {
        super(element, searchCallback, null);

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
        this.clear();
    }
}

async function handleTaskChange(event) {
    let checked = event.target.checked;

    let group = event.target.closest(".group").dataset.id;
    let task = event.target.closest(".task").dataset.id;

    try {
        let url = "/api/group/" + group + "/completed/" + task;
        let options = {
            method: checked ? "PUT" : "DELETE"
        };

        let response = await myfetch(url, options);
        handleResponse(response);
    } catch(e) {
        toast("error", e);
        event.target.checked = !checked;
    }
}

async function handleElaborationChange(event) {
    let data = event.target.selectedOptions[0].dataset;
    let group_data = event.target.closest(".group").dataset;

    let group = group_data.id;
    let experiment = group_data.experiment;
    if(experiment === undefined) {
        experiment = event.target.closest(".experiment").dataset.id;
    }

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
        handleResponse(response);

        event.target.dataset.prev_selected = event.target.selectedIndex;
    } catch(e) {
        toast("error", e);
        event.target.selectedIndex = event.target.dataset.prev_selected;
    }
}

async function handleCommentSave(event) {
    let group = event.target.closest(".group").dataset.id;
    let comment = event.target.closest(".comment").querySelector("textarea").value;

    try {
        let url = "/api/group/" + group + "/comment";

        let response = await myfetch(url, {
            method: "PUT",
            headers: new Headers({"Content-Type": "application/json"}),
            body: JSON.stringify(comment)
        });
        handleResponse(response);

        event.target.closest(".comment").classList.remove("unsaved");
    } catch(e) {
        toast("error", e);
    }
}

function handleCommentDate(event) {
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
}

// Search/add students and delete them after confirmation
async function handleStudentClick(event) {
    let target = event.target;
    let group = target.closest(".group").dataset.id;

    if(!(target instanceof HTMLLIElement)) {
        return;
    }

    if(target.classList.contains("add")) {
        searchBox.activate(async student => {
            let node = document.createElement("li");
            node.textContent = student.name;
            node.dataset.id = student.id;
            node.dataset.instructed = student.instructed;
            event.target.closest("ul").appendChild(node);

            try {
                let url = "/api/group/" + group + "/student/" + student.id;

                let response = await myfetch(url, {
                    method: "PUT"
                });
                handleResponse(response);

                searchBox.deactivate();
            } catch(e) {
                toast("error", e);
                event.target.closest("ul").removeChild(node);
            }
        });
    } else {
        let studentId = target.dataset.id;
        let studentName = target.textContent;

        let warningMessage =
            studentName + " wirklich aus der Gruppe entfernen?\n" +
            "Dies ist nur möglich, falls die Gruppe noch keine Aufgaben " +
            "abgeschlossen oder Ausarbeitungen eingereicht hat!\n\n" +
            "Bei Gruppenwechseln oder dem Auflösen einer Gruppe müssen für " +
            "*ALLE* Teilnehmer der alten Gruppe neue Gruppen angelegt werden, " +
            "damit der Bestanden-Status am Ende korrekt berechnet werden kann. " +
            "Die alte Gruppe darf nicht weiter verwendet werden und kann mit " +
            "\"(ENDE)\" im Kommentarfeld ausgegraut und ans Ende gestellt werden.";
        if(!confirm(warningMessage)) {
            return;
        }

        let parent = target.parentNode;
        parent.removeChild(target);

        try {
            let url = "/api/group/" + group + "/student/" + studentId;

            let response = await myfetch(url, {
                method: "DELETE"
            });
            handleResponse(response, {
                422: "Es existieren bereits abgeschlossene Aufgaben oder Ausarbeitungen!"
            });

            searchBox.deactivate();
        } catch(e) {
            toast("error", e);
            parent.appendChild(target);
        }
    }
}

async function searchStudents(terms) {
    let year = parseInt(document.body.dataset.year);
    if(!year) {
        throw "Invalid or no year found";
    }

    let response = await myfetch("/api/student/search", {
        method: "POST",
        headers: new Headers({"Content-Type": "application/json"}),
        body: JSON.stringify({
            terms: terms,
            year: year
        })
    });
    handleResponse(response);

    let students = await response.json();
    let elements = students.map(student => {
        return {
            text: student.name,
            data: {
                id: student.id,
                name: student.name,
                instructed: student.instructed
            }
        }
    });

    return Promise.resolve(elements);
}
