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
});
