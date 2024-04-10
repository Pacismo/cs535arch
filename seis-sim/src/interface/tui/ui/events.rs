use super::*;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

impl<'a> Runtime<'a> {
    pub fn process_input(&mut self) -> Result<bool, Box<dyn Error>> {
        if self.running.enabled {
            self.running_event_handler()
        } else {
            self.handle_event()
        }
    }

    fn running_event_handler(&mut self) -> Result<bool, Box<dyn Error>> {
        if event::poll(if self.running.slow_mode {
            Duration::from_millis(100)
        } else {
            Duration::from_nanos(0)
        })? {
            if matches!(
                event::read()?,
                Event::Key(KeyEvent {
                    kind: KeyEventKind::Press,
                    code: KeyCode::Esc,
                    ..
                })
            ) {
                self.running.enabled = false;

                return Ok(true);
            }
        }

        if self.clocks_required != 0 {
            self.clocks += self.clocks_required;
            self.clocks_required = match self.pipeline.clock(self.clocks_required) {
                libpipe::ClockResult::Stall(clocks) => clocks,
                libpipe::ClockResult::Flow => 1,
                libpipe::ClockResult::Dry => self.pipeline.memory_module().wait_time(),
            };
        } else {
            self.running.enabled = false;
        }

        Ok(true)
    }

    fn handle_event(&mut self) -> Result<bool, Box<dyn Error>> {
        match event::read()? {
            Event::Key(key) => match key.kind {
                KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => Ok(false),

                    KeyCode::Char('1') => {
                        self.view = View::Registers;
                        Ok(true)
                    }
                    KeyCode::Char('2') => {
                        self.view = View::Memory;
                        Ok(true)
                    }
                    KeyCode::Char('3') => {
                        self.view = View::Cache;
                        Ok(true)
                    }
                    KeyCode::Char('4') => {
                        self.view = View::PipelineStages;
                        Ok(true)
                    }

                    KeyCode::Tab => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            self.view = self.view.previous()
                        } else {
                            self.view = self.view.next();
                        }

                        Ok(true)
                    }

                    KeyCode::Up => {
                        if matches!(self.view, View::Memory) {
                            self.memory_view.page_offset =
                                self.memory_view.page_offset.saturating_sub(1).clamp(
                                    0,
                                    PAGE_SIZE / if self.memory_view.disassembly { 4 } else { 8 },
                                );
                        } else if matches!(self.view, View::Cache) {
                            self.cache_view.scroll = self.cache_view.scroll.saturating_sub(1);
                        }

                        Ok(true)
                    }
                    KeyCode::Down => {
                        if matches!(self.view, View::Memory) {
                            self.memory_view.page_offset =
                                self.memory_view.page_offset.saturating_add(1).clamp(
                                    0,
                                    PAGE_SIZE / if self.memory_view.disassembly { 4 } else { 8 },
                                );
                        } else if matches!(self.view, View::Cache) {
                            self.cache_view.scroll = self.cache_view.scroll.saturating_add(1);
                        }

                        Ok(true)
                    }

                    KeyCode::PageUp => {
                        if matches!(self.view, View::Memory) {
                            self.memory_view.page_offset =
                                self.memory_view.page_offset.saturating_sub(16).clamp(
                                    0,
                                    PAGE_SIZE / if self.memory_view.disassembly { 4 } else { 8 },
                                );
                        }

                        Ok(true)
                    }
                    KeyCode::PageDown => {
                        if matches!(self.view, View::Memory) {
                            self.memory_view.page_offset =
                                self.memory_view.page_offset.saturating_add(16).clamp(
                                    0,
                                    PAGE_SIZE / if self.memory_view.disassembly { 4 } else { 8 },
                                );
                        }

                        Ok(true)
                    }

                    KeyCode::Left => {
                        if matches!(self.view, View::Memory) {
                            self.memory_view.page_offset = 0;
                            self.memory_view.page =
                                self.memory_view.page.saturating_sub(1).clamp(0, PAGES - 1);
                        } else if matches!(self.view, View::Cache) {
                            self.cache_view.index = self.cache_view.index.saturating_sub(1);
                        } else if matches!(self.view, View::PipelineStages) {
                            self.pipeline_view.stage = self.pipeline_view.stage.saturating_sub(1);
                        }

                        Ok(true)
                    }
                    KeyCode::Right => {
                        if matches!(self.view, View::Memory) {
                            self.memory_view.page_offset = 0;
                            self.memory_view.page =
                                self.memory_view.page.saturating_add(1).clamp(0, PAGES - 1);
                        } else if matches!(self.view, View::Cache) {
                            self.cache_view.index = self.cache_view.index.saturating_add(1);

                            if self.cache_view.index >= self.cache_view.count {
                                self.cache_view.index = self.cache_view.count - 1;
                            }
                        } else if matches!(self.view, View::PipelineStages) {
                            self.pipeline_view.stage = (self.pipeline_view.stage + 1)
                                .clamp(0, 5 - PIPELINE_STAGE_VIEW_COUNT)
                        }

                        Ok(true)
                    }

                    KeyCode::Char('d') => {
                        if matches!(self.view, View::Memory) {
                            self.memory_view.disassembly = !self.memory_view.disassembly;
                        }

                        if self.memory_view.disassembly {
                            if self.memory_view.binary {
                                self.memory_view.page_offset *= BYTES_PER_ROW_BINARY / 4;
                            } else {
                                self.memory_view.page_offset *= BYTES_PER_ROW_HEXDEC / 4;
                            }
                        } else {
                            if self.memory_view.binary {
                                self.memory_view.page_offset /= BYTES_PER_ROW_BINARY / 4;
                            } else {
                                self.memory_view.page_offset /= BYTES_PER_ROW_HEXDEC / 4;
                            }
                        }

                        Ok(true)
                    }

                    KeyCode::Char('b') => {
                        if matches!(self.view, View::Memory) {
                            self.memory_view.binary = !self.memory_view.binary;
                        }

                        if !self.memory_view.disassembly {
                            if self.memory_view.binary {
                                self.memory_view.page_offset *=
                                    (BYTES_PER_ROW_HEXDEC / BYTES_PER_ROW_BINARY).max(1);
                            } else {
                                self.memory_view.page_offset /=
                                    (BYTES_PER_ROW_HEXDEC / BYTES_PER_ROW_BINARY).max(1)
                            }
                        }

                        Ok(true)
                    }

                    KeyCode::Char('c') => {
                        if self.clocks_required != 0 {
                            self.clocks_required = match self.pipeline.clock(1) {
                                libpipe::ClockResult::Stall(clocks) => clocks,
                                libpipe::ClockResult::Flow => 1,
                                libpipe::ClockResult::Dry => {
                                    self.pipeline.memory_module().wait_time()
                                }
                            };
                            self.clocks += 1;
                        }
                        Ok(true)
                    }

                    KeyCode::Char('f') => {
                        if self.clocks_required != 0 {
                            self.clocks += self.clocks_required;
                            self.clocks_required = match self.pipeline.clock(self.clocks_required) {
                                libpipe::ClockResult::Stall(clocks) => clocks,
                                libpipe::ClockResult::Flow => 1,
                                libpipe::ClockResult::Dry => {
                                    self.pipeline.memory_module().wait_time()
                                }
                            };
                        }

                        Ok(true)
                    }

                    KeyCode::Char('r') => {
                        self.running.enabled = true;
                        self.running.progress_bar = 0;
                        self.running.slow_mode = false;

                        Ok(true)
                    }

                    KeyCode::Char('s') => {
                        self.running.enabled = true;
                        self.running.progress_bar = 0;
                        self.running.slow_mode = true;

                        Ok(true)
                    }

                    _ => Ok(true),
                },

                _ => Ok(true),
            },

            _ => Ok(true),
        }
    }
}
