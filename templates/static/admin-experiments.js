document.addEventListener("DOMContentLoaded", () => {
    for(let tasks of document.querySelectorAll("ol.tasks")) {
        tasks.addEventListener("click", onTaskClick);
    }

    for(let experiment of document.querySelectorAll(".experiment h2")) {
        experiment.addEventListener("click", onDeleteExperiment);
    }

    document.querySelector("#add-experiment")
        .addEventListener("click", onNewExperiment);
});

async function onTaskClick(event) {
    let target = event.target;

    if(!(target instanceof HTMLLIElement)) {
        return;
    }

    let experiment = target.closest(".experiment").dataset.id;
    let parent = target.parentNode;

    if(target.classList.contains("add")) {
        let task = prompt("Name der neuen Aufgabe (z. B. 2b oder Z1 bei Zusatzaufgaben):");
        if(task === null) {
            return;
        }

        try {
            let url = "/api/experiment/" + experiment + "/task";

            let response = await myfetch(url, {
                method: "POST",
                headers: new Headers({"Content-Type": "application/json"}),
                body: JSON.stringify(task.trim())
            });
            if(!response.ok) {
                throw "API error";
            }

            let id = await response.json();

            let node = document.createElement("li");
            node.textContent = task;
            node.dataset.id = id;
            insertSorted(parent, node, document.createTextNode("\n"));
        } catch(e) {
            toast("error", e);
        }
    } else {
        let taskId = target.dataset.id;
        let taskName = target.textContent;

        if(!confirm(taskName + " wirklich löschen?")) {
            return;
        }

        parent.removeChild(target);

        try {
            let url = "/api/experiment/" + experiment + "/task/" + taskId;

            let response = await myfetch(url, {
                method: "DELETE"
            });
            if(!response.ok) {
                throw "API error";
            }
        } catch(e) {
            toast("error", e);
            insertSorted(parent, target, document.createTextNode("\n"));
        }
    }
}

async function onDeleteExperiment(event) {
    let target = event.target;

    let experiment = target.closest(".experiment");
    let experimentId = experiment.dataset.id;
    let experimentName = experiment.querySelector("h2").textContent;

    if(!confirm(experimentName + " wirklich löschen?")) {
        return;
    }

    try {
        let url = "/api/experiment/" + experimentId;

        let response = await myfetch(url, {
            method: "DELETE"
        });
        if(!response.ok) {
            throw "API error";
        }

        experiment.parentNode.removeChild(experiment);
    } catch(e) {
        toast("error", e);
    }
}

async function onNewExperiment() {
    let experiment = prompt("Name des neuen Versuches (z. B. Versuch 3):");
    if(experiment === null) {
        return;
    }

    let year = parseInt(document.body.dataset.year);

    try {
        let url = "/api/experiment";

        let response = await myfetch(url, {
            method: "POST",
            headers: new Headers({"Content-Type": "application/json"}),
            body: JSON.stringify({
                name: experiment.trim(),
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

function insertSorted(parent, node, whitespace, compare) {
    if(compare === undefined) {
        compare = (left, right) => {
            return left.textContent.localeCompare(right.textContent);
        };
    }

    // simple linear insertion sort
    for(var i = 0; i < parent.children.length; ++i) {
        if(compare(parent.children[i], node) > 0) {
            parent.insertBefore(node, parent.children[i]);
            if(whitespace !== undefined) {
                parent.insertBefore(whitespace, node.nextSibling);
            }
            return;
        }
    }

    // larger than all other elements
    if(whitespace !== undefined) {
        parent.appendChild(whitespace);
    }
    parent.appendChild(node);
}
