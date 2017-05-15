document.addEventListener("DOMContentLoaded", () => {
    for (input of document.querySelectorAll(".task input")) {
        input.addEventListener("change", async (event) => {
            let checked = event.target.checked;
            let ids = event.target.id.split("-");

            try {
                let url = "/api/completed/" + ids[1] + "/" + ids[2];
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
                console.log("Error changing " + event.target.id + " completion: " + e);
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
                    response = await fetch(url, {
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
});
