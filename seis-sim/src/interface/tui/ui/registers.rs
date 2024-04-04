use super::*;

impl<'a> Runtime<'a> {
    pub fn draw_registers_view(&mut self, chunks: &Rc<[Rect]>, buf: &mut Buffer) {
        let lines = chunks[1].height as usize;
        let splits = Layout::horizontal([Constraint::Min(16); 6]).split(chunks[1]);

        self.pipeline
            .registers()
            .iter()
            .enumerate()
            .map(|(i, val)| {
                Line::from(vec![
                    format!("{:>3}: ", get_name(i as Register).unwrap()).into(),
                    if i as Register == PC {
                        format!("{val:#010X}").blue().bold()
                    } else {
                        format!("{val}").blue().bold()
                    },
                ])
            })
            .collect::<Vec<_>>()
            .chunks(lines)
            .enumerate()
            .for_each(|(i, x)| List::new(x.into_iter().map(|x| x.clone())).render(splits[i], buf));
    }
}
