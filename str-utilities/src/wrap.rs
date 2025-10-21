pub struct WrappingPoints<'a> {
    on: &'a str,
    splitter: &'a [char],
    max_width: usize,
    /// whether to break before or after
    after: bool,
    current: usize,
}

pub static WHITESPACE: &[char] = &[' ', '\t', '\n'];

impl<'a> WrappingPoints<'a> {
    pub fn new(on: &'a str, max_width: usize, after: bool) -> Self {
        Self {
            splitter: WHITESPACE,
            on,
            max_width,
            after,
            current: 0,
        }
    }

    pub fn new_custom_splitter(
        on: &'a str,
        max_width: usize,
        after: bool,
        splitter: &'a [char],
    ) -> Self {
        Self {
            splitter,
            on,
            max_width,
            after,
            current: 0,
        }
    }
}

impl<'a> Iterator for WrappingPoints<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let rest = &self.on.get(self.current..)?;
        let mut last = 0;
        for (idx, matched) in rest.match_indices(self.splitter) {
            if idx >= self.max_width && last > 0 {
                let from = if self.after { idx } else { last };
                let index = self.current + from;
                self.current += from + matched.len();
                return Some(index);
            }

            if let "\n" = matched
                && !rest[..idx].trim_start().is_empty()
            {
                self.current += idx + matched.len();
                return Some(self.current);
            }

            last = idx;
        }
        self.current += rest.len();
        None
    }
}

pub fn wrap_text(on: &str, max_width: usize) -> String {
    wrap_text_options(on, max_width, false, "", WHITESPACE)
}

pub fn wrap_text_options(
    on: &str,
    max_width: usize,
    after: bool,
    prefix: &str,
    splitter: &[char],
) -> String {
    let mut buf = String::new();
    let points = WrappingPoints::new_custom_splitter(on, max_width, after, splitter);
    let mut last = 0;
    for point in points {
        buf.push_str(prefix);
        buf.push_str(on[last..point].trim_start());
        buf.push('\n');
        last = point + 1;
    }

    if let Some(part) = on.get(last..) {
        buf.push_str(prefix);
        buf.push_str(part.trim_start());
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    static INPUT: &str = "Lorem ipsum dolor sit amet, ius no alia volutpat repudiare, delectus adolescens rationibus ex usu. Quo iudico nusquam qualisque ea, mei regione commune insolens cu, sed habeo erant constituam in. Dolor electram cu eum. Has an illum eleifend philosophia, ea mel regione tamquam facilis. No cibo nobis dignissim sed. Te sea quem tritani, scaevola appetere no eos.

His harum reformidans philosophia in, has laudem patrioque prodesset id, nonumes apeirian efficiendi vim at. Vide dicta omnium et sit, ad atomorum honestatis vis. Ea aeque albucius ius, et deseruisse efficiantur pro, ut per malorum insolens. Et cum accusamus patrioque, at sit stet laboramus, per ad debet corrumpit. Rebum iudicabit per eu, ea tota novum deserunt est.

Eligendi euripidis necessitatibus cu per, wisi minimum vel an. In mei veri nusquam quaerendum, sed eruditi lucilius theophrastus ad. Duo vitae possit viderer at, ut sit elitr signiferumque. Mea an veri facilisis disputationi, cu eum antiopam neglegentur. Justo iudico eam cu, pri graecis patrioque abhorreant eu.

Eros percipit convenire nec te, ne sea numquam inermis, ut sea decore accusata inciderint. Id eum delectus legendos maiestatis, solet mollis eu mei. Repudiare scripserit eu vim, iudicabit urbanitas repudiandae per id. Ei saepe utamur facilis pri, wisi mollis eum te, eum nostro definitiones eu. At quot gubergren adversarium vel, no sit urbanitas complectitur, te his feugiat volumus facilisis. Eum persius ullamcorper an, vis cu luptatum definitiones.

Ad vis populo mollis, vis cu democritum definitionem. Ad quo enim quaeque scripserit, sit aperiam volumus splendide ut, in dicit commune deleniti sit. Ne quo veri eripuit. Posse nominati similique at mea.";

    #[test]
    fn basic() {
        let max_width = 40;
        let points = WrappingPoints::new(INPUT, max_width, false);
        let mut last = 0;
        for point in points {
            assert!(
                point - last <= max_width,
                "found difference {difference}",
                difference = point - last
            );
            last = point;
        }
    }

    #[test]
    fn lines() {
        let max_width = 40;
        let out = wrap_text(INPUT, max_width);
        for line in out.lines() {
            assert!(
                line.len() <= max_width,
                "found line of length {length}",
                length = line.len()
            );
        }
    }
}
