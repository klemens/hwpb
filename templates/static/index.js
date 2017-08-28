document.addEventListener("DOMContentLoaded", () => {
    new SearchBox(document.querySelector(".search"), searchGroups, null);
});

async function searchGroups(terms) {
    let response = await myfetch("/api/group/search", {
        method: "POST",
        headers: new Headers({"Content-Type": "application/json"}),
        body: JSON.stringify(terms)
    });
    if(!response.ok) {
        throw "API error";
    }

    let groups = await response.json();
    let elements = groups.map(group => {
        let students = " (" + group.students.map(s => s.name).join(", ") + ")";
        return {
            name: "Gruppe " + group.desk + ", " + group.day + students,
            href: "/group/" + group.id
        };
    });

    return Promise.resolve(elements);
}