wasm_bindgen('/pkg/package_bg.wasm')
    .then(registerEvents)
    .catch(console.error);

function clearData() {
    wasm_bindgen.clear_data();
    document.getElementById("results").innerText = "";
}

function gatherData() {
    let goal = document.getElementById("goal").value;
    let count = +document.getElementById("goal_count").value;
    if (goal == "" || count <= 0) {
        return;
    }
    let num_runs = 100;
    let now = window.performance.now();
    while (window.performance.now() - now < 500) {
        wasm_bindgen.run(num_runs, goal, count);
        num_runs *= 2;
    }
    document.getElementById("results").innerText = wasm_bindgen.results();
}

function registerEvents() {
    wasm_bindgen.init_banner(1, 1, 1, 1, 3, 3);
    document.getElementById("run").addEventListener("click", gatherData);

    document.getElementById("goal").addEventListener("input", clearData);
    document.getElementById("goal_count").addEventListener("input", clearData);
}