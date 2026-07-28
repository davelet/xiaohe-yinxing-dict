use super::super::*;

pub(crate) fn render_readme(ui: &mut Ui, nav: &mut HelpNav) {
    h1(ui, "小鹤音形帮助文档");
    label_hl_mk(ui, "小鹤音形", |s| RichText::new(s).strong());
    qt(ui, "一个简单易学高效的输入方案");
    sp(ui);

    h2(ui, "指引");
    p(ui, "这里是一些基础知识，指引你做好学前准备。");
    ui.horizontal(|ui| {
        label_hl(ui, "请先看");
        lnk(ui, nav, "学习指引", "vy");
        label_hl(ui, "，了解详情。");
    });
    sp(ui);

    h2(ui, "入门");
    ui.horizontal_wrapped(|ui| {
        label_hl(ui, "当你准备好，就可进入");
        lnk(ui, nav, "入门", "xh");
        label_hl(ui, "学习了，");
        lnk(ui, nav, "小鹤双拼", "up");
        label_hl(
            ui,
            "方案内置各大拼音输入法，在其设置中选中即可，在使用中记忆键位，通常一周就能适应。",
        );
    });
    ui.horizontal_wrapped(|ui| {
        label_hl(ui, "如果你想进阶");
        lnk(ui, nav, "小鹤音形", "ux");
        label_hl(
            ui,
            "，可以在使用小鹤双拼一周后开始，小鹤音形的音部即双拼，循序渐进能让学习曲线更加平滑。",
        );
    });
    sp(ui);

    h2(ui, "应用");
    ui.horizontal_wrapped(|ui| {
        label_hl(ui, "小鹤音形的学习就要接触到小鹤音形");
        lnk(ui, nav, "输入法", "yy");
        label_hl(
            ui,
            "，这是一个独立的输入法，需要在小鹤官网下载安装使用。同时你需要对",
        );
        lnk(ui, nav, "win版", "pc");
        label_hl(ui, "、");
        lnk(ui, nav, "安卓版", "sj");
        label_hl(ui, "输入法的功能有所了解，那就是这部分的内容。");
    });
    sp(ui);

    h2(ui, "捐赠");
    ui.horizontal_wrapped(|ui| {
        label_hl(ui, "如果你觉得小鹤双拼&小鹤音形对你打字速度的提高有所帮助，或者想对小鹤的持续发展提供一点支持，欢迎给我们");
        lnk(ui, nav, "捐赠", "gy");
    });
    sp(ui);

    h2(ui, "交流");
    ui.horizontal_wrapped(|ui| {
        label_hl(ui, "可到小鹤QQ频道或QQ群 ");
        ext_link(ui, "182883808", "tencent://message/?uin=182883808");
        label_hl(ui, " 交流和反馈问题。");
    });

    hr(ui);
    navrow(ui, nav, None, Some(("1 入门", "xh")));
}
