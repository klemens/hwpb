document.addEventListener("DOMContentLoaded", () => {
    document.querySelector("header select")
        .addEventListener("change", onYearChange);
});

async function onYearChange(event) {
    let target = event.target;

    if(target.value === "new-year") {
        let year = parseInt(prompt("Neues Jahr:"));
        if(year === null) {
            return;
        }

        try {
            let url = "/api/year/" + year;

            let response = await myfetch(url, {
                method: "PUT"
            });
            if(!response.ok) {
                throw "API error";
            }

            location = "/admin/" + year;
        } catch(e) {
            toast("error", e);
        }
    } else {
        let site = document.body.dataset.site;
        location = "/admin/" + target.value + "/" + site;
    }
}
