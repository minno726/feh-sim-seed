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
    document.getElementById("results_pct").innerHTML = "";
    document.getElementById("run").innerText = "Run";
    document.getElementById("graph_line").setAttribute("d", "");
    document.getElementById("graph_highlights").innerHTML = "";
    document.getElementById("graph_sample_count").innerHTML = "";
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
    document.getElementById("run").innerText = "More";
    drawGraph();
}

function drawGraph() {
    let percentiles = Array(1000);
    for (let i = 0; i < percentiles.length; ++i) {
        percentiles[i] = i / percentiles.length;
    }
    let results = wasm_bindgen.results(percentiles);
    let xmin = 0;
    let ymin = 0;
    let xmax = 100;
    let ymax = 60;
    let x = (pct) => pct * (xmax - xmin) + xmin;
    let y = (val) => ymax - (val / results[results.length - 1] * (ymax - ymin) + ymin);
    let path = `M ${x(0)} ${y(results[0])} `;
    for (let i = 1; i < results.length; ++i) {
        if (results[i] != results[i - 1]) {
            path += `L ${x(percentiles[i])} ${y(results[i])} `;
        }
    }

    document.getElementById("graph_highlights").innerHTML = "";
    let highlight_percentiles = [0.25, 0.5, 0.75, 0.9, 0.99];
    highlights = wasm_bindgen.results(highlight_percentiles);
    for (let i = 0; i < highlights.length; ++i) {
        let dot = document.createElementNS("http://www.w3.org/2000/svg", "circle");
        dot.setAttribute("cx", x(highlight_percentiles[i]));
        dot.setAttribute("cy", y(highlights[i]));
        dot.setAttribute("r", "0.75px");

        let text = document.createElementNS("http://www.w3.org/2000/svg", "text");
        text.innerHTML = `${highlight_percentiles[i] * 100}%: ${highlights[i]}`;
        text.setAttribute("dx", x(highlight_percentiles[i]) - 1);
        text.setAttribute("dy", y(highlights[i]) - 1);
        text.setAttribute("text-anchor", "end");
        text.setAttribute("font-size", "15%");
        document.getElementById("graph_highlights").appendChild(dot);
        document.getElementById("graph_highlights").appendChild(text);
    }

    let sample_count = wasm_bindgen.num_trials();
    document.getElementById("graph_sample_count").innerHTML = `${sample_count} samples`;

    document.getElementById("graph_line").setAttribute("d", path);
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