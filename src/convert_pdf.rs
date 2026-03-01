use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    process::{Command, Stdio},
};

fn main() {
    let edition = env::args()
        .nth(1)
        .expect("Pass the edition date as the first argument");

    let pdf = format!("pdfs/{edition}.pdf");

    fs::create_dir_all(format!("svgs/{edition}")).expect("Failed to create dir");

    // convert to pdf
    {
        let res = Command::new("pdf2svg")
            .args([&pdf, &format!("svgs/{edition}/%d.svg"), "all"])
            .spawn()
            .expect("Failed to spawn pdf2svg")
            .wait()
            .expect("Failed to run pdf2svg");
        if !res.success() {
            panic!("pdf2svg exited with status code {:?}", res.code());
        }
    }

    let page_heights: Vec<f32> = {
        let output = Command::new("pdfcpu")
            .args(["boxes", "list", &pdf])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn pdfcpu")
            .wait_with_output()
            .expect("Failed to run pdfcpu");
        if !output.status.success() {
            panic!("pdf2svg exited with status code {:?}", output.status.code());
        }

        let string = String::from_utf8_lossy(&output.stdout);

        string
            .lines()
            .filter(|line| line.trim().starts_with("MediaBox"))
            .map(|line| {
                line.split_whitespace()
                    .find_map(|part| part.strip_prefix("h="))
                    .unwrap()
                    .parse()
                    .unwrap()
            })
            .collect()
    };

    let output = Command::new("pdfcpu")
        .args(["annotations", "list", &pdf])
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pdfcpu")
        .wait_with_output()
        .expect("Failed to run pdfcpu");
    if !output.status.success() {
        panic!("pdf2svg exited with status code {:?}", output.status.code());
    }

    let string = String::from_utf8_lossy(&output.stdout);

    let n_annotations: u16 = string["optimizing...\n".len()..]
        .split_once(' ')
        .unwrap()
        .0
        .parse()
        .unwrap();
    if n_annotations == 0 {
        return;
    }

    for page in string.split("\nPage ").skip(1) {
        let page_num: u16 = page.split_once(':').unwrap().0.parse().unwrap();

        let links = page.lines().skip(5).map(|line| {
                let parts = line.split('│').skip(2).map(str::trim).collect::<Vec<_>>();

                let content = parts[1]
                    .replace('&', "&amp;")
                    .replace('"', "&quot;")
                    .replace('\'', "&apos;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");

                let target = if content.starts_with("https://") {
                    "target=\"_blank\""
                } else {
                    ""
                };

                let rect = parts[0][1..parts[0].len() - 1]
                    .split(", ")
                    .map(str::trim)
                    .map(str::parse)
                    .map(Result::unwrap)
                    .collect::<Vec<u16>>();

                let x_min = rect[0].min(rect[2]);
                let x_max = rect[0].max(rect[2]);

                let y_min = rect[1].min(rect[3]);
                let y_max = rect[1].max(rect[3]);

                let width = x_max - x_min;
                let height = y_max - y_min;

                format!("
                <a xlink:href=\"{content}\" href=\"{content}\" {target}>
                    <rect x=\"{x_min}\" y=\"{y_min}\" width=\"{width}\" height=\"{height}\" fill=\"white\" fill-opacity=\"0\" stroke=\"none\" pointer-events=\"all\"/>
                </a>")
            }).collect::<String>();

        let file = format!("svgs/{edition}/{page_num}.svg");
        let contents = fs::read_to_string(&file).expect("Failed to read file");
        let trimmed = contents
            .trim()
            .strip_suffix("</svg>")
            .expect("Svg file doesn't end in </svg>");

        let page_height = page_heights[page_num as usize - 1];

        let svg = format!(
            "{trimmed}
                <g id=\"link-layer\" transform=\"translate(0, {page_height}) scale(1, -1)\">
                    {links}
                </g>
            </svg>"
        );

        fs::write(file, svg).expect("Failed to write to file");
    }
}
