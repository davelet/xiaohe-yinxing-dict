use super::super::*;

pub(crate) fn render_zg(ui: &mut Ui, nav: &mut HelpNav, _manager: &HelpManager) {
    h2(ui, "1.2.2 鹤形：字根");

    h4(ui, "一、字根合图");
    ui.horizontal_wrapped(|ui| {
        label_hl(
            ui,
            "字根合图可点击主界面「部件字根键位图」按钮查看，或参考小鹤官网 ",
        );
        ext_link(ui, "flypy.cc", "https://flypy.cc");
        label_hl(ui, " 上的图示。");
    });
    sp(ui);

    h4(ui, "二、笔画");
    red(ui, "拆分最小单元，笔画：横A 竖L 撇P 点D 折V 捺N");
    sp(ui);
    tbl(
        ui,
        70.0,
        &["键位", "笔画", "表达"],
        &[
            &["a", "横", "一"],
            &["l", "竖", "丨"],
            &["p", "撇", "丿"],
            &["d", "点", "丶"],
            &["v", "折", "乛"],
            &["n", "捺", "乀"],
        ],
    );
    sp(ui);

    h4(ui, "三、部件字根");
    red(ui, "基本为偏旁部首，日常称谓定其键位，必须掌握！");
    sp(ui);
    for (k, parts, mem) in BUJIAN {
        ui.horizontal(|ui| {
            label_hl_mk(ui, k, |s| {
                RichText::new(s).monospace().strong().color(ORANGE)
            });
            label_hl(ui, ":");
            label_hl(ui, parts);
            if !mem.is_empty() {
                label_hl(ui, "—");
                ui.weak(*mem);
            }
        });
    }
    sp(ui);

    h4(ui, "四、小字字根");
    red(ui, "小字规则字根，免记忆");
    p(ui, "小字字根所在键列表（理解小字规则的用户请跳过此表）：");
    bul(
        ui,
        "一些可能会不认识的小字：戊wù、戌xū、耒lěi、爿pán、豕shǐ、臾yú、聿yù、廿niàn、巳sì、曳yè、夬guài",
    );
    sp(ui);
    for (k, chars) in XIAOZI {
        ui.horizontal(|ui| {
            label_hl_mk(ui, k, |s| {
                RichText::new(s).monospace().strong().color(ORANGE)
            });
            label_hl(ui, ":");
            label_hl(ui, chars);
        });
    }
    sp(ui);

    h4(ui, "五、拆分例字");
    ui.horizontal_wrapped(|ui| {
        label_hl(
            ui,
            "理解下面单字的拆分，就基本掌握鹤形字根了，也可到 拆分学习 ",
        );
        ext_link(ui, "https://flypy.cc/if", "https://flypy.cc/if");
        label_hl(ui, " 页面进行拆分练习");
    });
    sp(ui);
    tbl4(ui, 70.0, &["例字", "全码", "首形", "末形"], CHAIFEN);

    hr(ui);
    navrow(ui, nav, _manager, Some("gz"), Some("yy"));
}
