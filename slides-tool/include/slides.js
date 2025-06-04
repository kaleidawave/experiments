const slides = document.body.querySelector('#slides');
slides.firstElementChild.classList.add('current');
let slides_idx = 0;
const num_slides = slides.children.length;

document.title = `1/${num_slides}`;

function changeSlide(to) {
    slides.children[slides_idx].classList.remove('current');
    slides.children[to].classList.add('current');
    slides_idx = to;
    document.title = `${to + 1}/${num_slides}`;
}

function nextSlide() {
    changeSlide(Math.min(slides_idx + 1, num_slides - 1))
}

function prevSlide() {
    changeSlide(Math.max(slides_idx - 1, 0))
}

document.body.querySelector('#next').addEventListener('click', nextSlide);
document.body.querySelector('#prev').addEventListener('click', prevSlide);

document.body.addEventListener("keydown", (ev) => {
    if (ev.code === "ArrowRight") nextSlide()
    else if (ev.code === "ArrowLeft") prevSlide()
});

export function sleep(ms = 1000) {
    return new Promise((cb, _) => setTimeout(cb, ms))
}

async function* sequence(interval, count) {
    for (let i = 0; i < count; i++) {
        await sleep(interval);
        yield i;
    }
}

if (location.hash === "#automate") {
    const interval = 20_000;
    const progressBar = document.createElement("div");
    progressBar.classList.add("progress");
    const anim = [
        { width: "0%" },
        { width: "100%" },
    ];

    const timing = {
        duration: interval,
        iterations: Number.POSITIVE_INFINITY,
    };
    progressBar.animate(anim, timing);
    document.body.append(progressBar);

    for await (const i of sequence(interval, 31)) {
        nextSlide();
    }
}