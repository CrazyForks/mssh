use ratatui::{prelude::*, widgets::*};

/// 统一的滚动状态管理器
#[derive(Debug, Clone)]
pub struct ScrollManager {
    pub scroll_offset: usize,  // 当前滚动偏移量
    pub visible_items: usize,  // 可视区域能显示的项目数量
    pub total_items: usize,    // 总项目数量
    pub selected_index: usize, // 当前选中项索引
}

impl ScrollManager {
    /// 创建新的滚动管理器
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            visible_items: 0,
            total_items: 0,
            selected_index: 0,
        }
    }

    /// 设置可视区域大小
    pub fn set_visible_items(&mut self, visible_items: usize) {
        self.visible_items = visible_items;
        self.update_scroll_position();
    }

    /// 设置总项目数量
    pub fn set_total_items(&mut self, total_items: usize) {
        self.total_items = total_items;
        // 确保选中索引在有效范围内
        if self.selected_index >= total_items && total_items > 0 {
            self.selected_index = total_items - 1;
        }
        self.update_scroll_position();
    }

    /// 设置选中项索引
    pub fn set_selected_index(&mut self, index: usize) {
        if index < self.total_items {
            self.selected_index = index;
            self.update_scroll_position();
        }
    }

    // /// 移动到下一项
    // pub fn next_item(&mut self) {
    //     if self.total_items > 0 {
    //         self.selected_index = (self.selected_index + 1) % self.total_items;
    //         self.update_scroll_position();
    //     }
    // }

    // /// 移动到上一项
    // pub fn previous_item(&mut self) {
    //     if self.total_items > 0 {
    //         self.selected_index = if self.selected_index == 0 {
    //             self.total_items - 1
    //         } else {
    //             self.selected_index - 1
    //         };
    //         self.update_scroll_position();
    //     }
    // }

    /// 更新滚动位置，确保当前项目在可视区域内
    pub fn update_scroll_position(&mut self) {
        if self.visible_items == 0 || self.total_items == 0 {
            self.scroll_offset = 0;
            return;
        }

        // 确保当前项目在可视区域内
        if self.selected_index < self.scroll_offset {
            // 当前项目在可视区域上方，向上滚动
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.visible_items {
            // 当前项目在可视区域下方，向下滚动
            self.scroll_offset = self.selected_index.saturating_sub(self.visible_items - 1);
        }

        // 确保滚动偏移量不超过最大值
        let max_scroll_offset = self.total_items.saturating_sub(self.visible_items);

        if self.scroll_offset > max_scroll_offset {
            self.scroll_offset = max_scroll_offset;
        }
    }

    /// 获取滚动信息
    pub fn get_scroll_info(&self) -> (usize, usize, usize) {
        (self.scroll_offset, self.visible_items, self.total_items)
    }

    // /// 获取当前选中项在可视区域中的相对位置
    // pub fn get_relative_selected_index(&self) -> usize {
    //     if self.selected_index >= self.scroll_offset {
    //         self.selected_index - self.scroll_offset
    //     } else {
    //         0
    //     }
    // }
}

/// 渲染滚动条
pub fn render_scrollbar(f: &mut Frame, area: Rect, scroll_manager: &ScrollManager) {
    let (scroll_offset, visible_items, total_items) = scroll_manager.get_scroll_info();

    if total_items <= visible_items {
        return;
    }

    // 计算滚动条区域（右侧边栏）
    let scrollbar_area = Rect::new(area.right().saturating_sub(1), area.y, 1, area.height);

    // 计算最大滚动偏移量
    let max_scroll_offset = total_items - visible_items;

    // 确保滚动偏移量不超过最大值
    let adjusted_scroll_offset = std::cmp::min(scroll_offset, max_scroll_offset);

    // 创建ScrollbarState
    let mut scrollbar_state = ScrollbarState::new(total_items)
        .viewport_content_length(visible_items)
        .position(adjusted_scroll_offset);

    // 创建Scrollbar widget
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .thumb_symbol("█")
        .track_symbol(Some("│"));

    // 渲染滚动条
    f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
}

// /// 渲染可滚动内容（表单专用）
// pub fn render_scrollable_form<F>(
//     f: &mut Frame,
//     area: Rect,
//     scroll_manager: &ScrollManager,
//     mut render_field: F,
//     sections: Vec<(usize, &str, usize)>,
// ) where
//     F: FnMut(&mut Frame, Rect, usize),
// {
//     let (scroll_offset, visible_items, _total_items) = scroll_manager.get_scroll_info();

//     let mut current_y = area.y;
//     let mut rendered_items = 0;

//     // 带段落标题的滚动渲染
//     for (section_start, section_title, section_item_count) in sections {
//         // 检查这个段落是否在可视区域内
//         let section_end = section_start + section_item_count;

//         if section_end <= scroll_offset {
//             // 整个段落在可视区域上方，跳过
//             continue;
//         }

//         if section_start >= scroll_offset + visible_items {
//             // 整个段落在可视区域下方，跳过
//             break;
//         }

//         // 计算段落在可视区域内的项目
//         let visible_start = std::cmp::max(section_start, scroll_offset);
//         let visible_end = std::cmp::min(section_end, scroll_offset + visible_items);

//         // 渲染段落标题（如果段落开头在可视区域内）
//         if section_start >= scroll_offset {
//             let title_height = 2;
//             let title_area = Rect::new(
//                 area.x,
//                 current_y,
//                 area.width,
//                 title_height,
//             );

//             let title_block = Block::default()
//                 .borders(Borders::NONE)
//                 .title(
//                     Span::styled(
//                         section_title,
//                         Style::default().fg(Color::Cyan)
//                     )
//                 )
//                 .title_alignment(Alignment::Center);

//             f.render_widget(title_block, title_area);
//             current_y += title_height;
//         }

//         // 渲染段落内的可见项目
//         for item_index in visible_start..visible_end {
//             if rendered_items >= visible_items {
//                 break;
//             }

//             let item_height = 3; // 每个项目3行高度
//             let item_area = Rect::new(
//                 area.x,
//                 current_y,
//                 area.width,
//                 item_height,
//             );

//             render_field(f, item_area, item_index);
//             current_y += item_height;
//             rendered_items += 1;
//         }
//     }

//     // 渲染滚动条
//     render_scrollbar(f, area, scroll_manager);
// }

// / 渲染可滚动列表（列表专用）
// pub fn render_scrollable_list<F>(
//     f: &mut Frame,
//     area: Rect,
//     scroll_manager: &ScrollManager,
//     mut render_item: F,
// ) where
//     F: FnMut(&mut Frame, Rect, usize, bool), // 添加选中状态参数
// {
//     let (scroll_offset, visible_items, total_items) = scroll_manager.get_scroll_info();

//     for i in 0..visible_items {
//         if scroll_offset + i >= total_items {
//             break;
//         }

//         let item_index = scroll_offset + i;
//         let is_selected = item_index == scroll_manager.selected_index;

//         let item_height = area.height / visible_items.max(1) as u16;
//         let item_area = Rect::new(
//             area.x,
//             area.y + (i as u16 * item_height),
//             area.width.saturating_sub(if scroll_manager.needs_scrolling() { 1 } else { 0 }),
//             item_height,
//         );

//         render_item(f, item_area, item_index, is_selected);
//     }

//     // 渲染滚动条
//     render_scrollbar(f, area, scroll_manager);
// }
