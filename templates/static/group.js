let pushServer = null;

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
});
