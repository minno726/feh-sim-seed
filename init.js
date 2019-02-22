wasm_bindgen('/pkg/package_bg.wasm')
    .then(registerEvents)
    .catch(console.error);

function initBanner() {
    let r_count = +document.getElementById("focus_count_r").value;
    let b_count = +document.getElementById("focus_count_b").value;
    let g_count = +document.getElementById("focus_count_g").value;
    let c_count = +document.getElementById("focus_count_c").value;
    let rates = document.getElementById("starting_rates").value;
    let focus_rate = rates.split(' ')[0];
    let fivestar_rate = rates.split(' ')[1];
    wasm_bindgen.init_banner(r_count, b_count, g_count, c_count, focus_rate, fivestar_rate);
    updateGoalList();
    clearData();
}

function updateGoalList() {
    let r_count = +document.getElementById("focus_count_r").value;
    let b_count = +document.getElementById("focus_count_b").value;
    let g_count = +document.getElementById("focus_count_g").value;
    let c_count = +document.getElementById("focus_count_c").value;
    let goal_selector = document.getElementById("goal");
    let curr_goal = +goal_selector.value;
    goal_selector.innerHTML = "";
    let labels = [
        { label: "Any 5*", value: 0, enabled: true },
        { label: "Any focus", value: 1, enabled: true },
        { label: "Red focus", value: 2, enabled: r_count > 0 },
        { label: "Blue focus", value: 3, enabled: b_count > 0 },
        { label: "Green focus", value: 4, enabled: g_count > 0 },
        { label: "Colorless focus", value: 5, enabled: c_count > 0 }
    ];
    for (let label of labels) {
        if (label.enabled) {
            let option = document.createElement("option");
            option.value = label.value;
            option.innerText = label.label;
            if (curr_goal == label.value) {
                option.selected = true;
            }
            goal_selector.appendChild(option);
        } else {
            // Reset if currently selected goal is not allowed by the new banner.
            if (curr_goal === label.value) {
                goal_selector.firstChild.selected = true;
            }
        }
    }
}

function clearData() {
    wasm_bindgen.clear_data();
    document.getElementById("results").innerText = "";
    document.getElementById("run").innerText = "Run";
}

function gatherData() {
    let goal = +document.getElementById("goal").value;
    let count = +document.getElementById("goal_count").value;
    let num_runs = 100;
    let now = window.performance.now();
    while (window.performance.now() - now < 500) {
        wasm_bindgen.run(num_runs, goal, count);
        num_runs *= 2;
    }
    let percentiles = [0.25, 0.5, 0.75, 0.9, 0.99];
    let results = wasm_bindgen.results(percentiles);
    let ul = document.createElement("ul");
    for (let i = 0; i < results.length; ++i) {
        let li = document.createElement("li");
        li.innerHTML = percentiles[i] * 100 + '%: ' + results[i];
        ul.appendChild(li);
    }
    document.getElementById("results").innerHTML = "Results:";
    document.getElementById("results").appendChild(ul);
    document.getElementById("run").innerText = "More";
}

function registerEvents() {
    document.getElementById("run").addEventListener("click", gatherData);

    let goal_inputs = ["goal", "goal_count"]
    let banner_inputs = ["starting_rates", "focus_count_r", "focus_count_b", "focus_count_g", "focus_count_c"];
    for (let id of goal_inputs.concat(banner_inputs)) {
        document.getElementById(id).addEventListener("input", () => {
            clearData(); updateGoalList()
        });
    }
    document.getElementById("starting_rates").addEventListener("input", (ev) => {
        // Convenient handling for legendary banners.
        if (ev.target.value === "8 0") {
            for (let letter of ["r", "b", "g", "c"]) {
                document.getElementById("focus_count_" + letter).value = 3;
            }
        }
    });
    for (let id of banner_inputs) {
        document.getElementById(id).addEventListener("input", () => {
            initBanner();
        });
    }

    initBanner();
}