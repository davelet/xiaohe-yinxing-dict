use super::super::*;

pub(crate) fn render_xh(ui: &mut Ui, nav: &mut HelpNav) {
    h1(ui, "1 入门概述");

    h4(ui, "一、小鹤音形");
    bul(ui, "单字以“双拼＋双形”组合的标准四码音形类输入方案");
    ui.horizontal(|ui| {
        label_hl(ui, "    ");
        blue(ui, "双拼");
        label_hl(
            ui,
            "：声母、韵母各用一个字母表示，一个汉字的音用两个字母表达",
        );
    });
    ui.horizontal(|ui| {
        label_hl(ui, "    ");
        blue(ui, "双形");
        label_hl(
            ui,
            "：根据拆分规则把一个汉字按字根拆分出两个部分，以区分同音字",
        );
    });
    sp(ui);
    ui.horizontal(|ui| {
        label_hl(ui, "    • 双拼初学者请先阅读");
        lnk(ui, nav, "《学习指引》", "vy");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "    • 音形初学者看完上面指引后，请从本页看起");
    });
    sp(ui);

    h4(ui, "二、编码构成");
    num(ui, "1", "单字全码：双拼 + 双形");
    tbl(
        ui,
        80.0,
        &["单字", "双拼", "双形", "全码"],
        &[
            &["小", "xn", "丨丶", "xnld"],
            &["鹤", "he", "丶鸟", "hedn"],
            &["音", "yb", "立日", "yblo"],
            &["形", "xk", "开丿", "xkkp"],
        ],
    );
    ui.horizontal_wrapped(|ui| {
        label_hl_mk(ui, "简码", |s| RichText::new(s).strong());
        label_hl(ui, "：未满四码即显的字为简码字");
    });
    label_hl(ui, "　　　例：x小、hed河，称一简字、三简字");
    ui.horizontal(|ui| {
        label_hl(ui, "　　　实际使用有简打简，一二简字列表见“");
        lnk(ui, nav, "简码篇", "jm");
        label_hl(ui, "”");
    });
    sp(ui);

    num(ui, "2", "构词规则：");
    tbl(
        ui,
        110.0,
        &["字数", "规则", "例词", "全码"],
        &[
            &["二字词", "首字前两码＋末字前两码", "双拼", "ulpb"],
            &["三字词", "前两字首码＋末字前两码", "输入法", "urfa"],
            &["四以上", "前三字首码＋末字首码", "他乡遇故知", "txyv"],
        ],
    );
    ui.horizontal_wrapped(|ui| {
        label_hl_mk(ui, "简码", |s| RichText::new(s).strong());
        label_hl(ui, "：取各字声母未满四码即显的词为简码词");
    });
    label_hl(ui, "　　　例：vd知道、yly越来越，称二简词、三简词");
    ui.horizontal(|ui| {
        label_hl(ui, "　　　二简词在记忆情况下使用，列表见“");
        lnk(ui, nav, "二简词", "jm");
        label_hl(ui, "”");
    });
    sp(ui);

    hr(ui);
    ui.horizontal_wrapped(|ui| {
        label_hl(ui, "小鹤官方 ");
        ext_link(ui, "flypy.cc", "https://flypy.cc");
        label_hl(ui, " 发布「小鹤音形」输入法");
    });
    bul(
        ui,
        "采用《通用规范汉字表》国发〔2013〕23号文规定用字，本表收字8105个",
    );
    bul(ui, "拼字输入方式作为补充，支持GB18030-2022");
    hr(ui);
    navrow(ui, nav, Some(("导读", "readme")), Some(("1.1 双拼", "up")));
}
