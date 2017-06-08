document.addEventListener("DOMContentLoaded", () => {
    for (input of document.querySelectorAll(".task input")) {
        input.addEventListener("change", async (event) => {
            let checked = event.target.checked;

            let group = event.target.closest(".group").dataset.id;
            let task = event.target.closest(".task").dataset.id;

            try {
                let url = "/api/group/" + group + "/completed/" + task;
                let options = {
                    method: checked ? "PUT" : "DELETE",
                    deadline: 4000
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
                    headers: new Headers({"Content-Type": "text/plain"}),
                    body: comment
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
});
