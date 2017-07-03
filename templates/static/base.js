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
                link.textContent = element.name;
                node.appendChild(link);
            } else {
                node.textContent = element.name;

                if(element.id) {
                    node.dataset.id = element.id;
                    node.dataset.name = element.name;
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
            this.successCallback(data.id, data.name);
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
