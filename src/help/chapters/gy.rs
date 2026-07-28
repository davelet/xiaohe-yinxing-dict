use super::super::*;

pub(crate) fn render_gy(ui: &mut Ui, nav: &mut HelpNav) {
    h2(ui, "6 关于小鹤");
    p(ui, "帮助我们，让小鹤飞得更高！");
    sp(ui);

    bul(ui, "出品年代：");
    p(ui, "    - 双拼方案：2006年4月3日初稿 - 2007年1月8日定稿");
    p(ui, "    - 双形方案：2007年5月6日初稿 - 2008年2月23日定稿");
    sp(ui);

    bul(ui, "发展历程：");
    p(ui, "    - 2010.03.25 搜狗拼音V5.0版内置小鹤双拼方案");
    p(ui, "    - 2010.05.07 QQ拼音V3.2版内置小鹤双拼方案");
    p(ui, "    - 2010.09.21 谷歌拼音V2.3.14.85版内置小鹤双拼方案");
    p(ui, "    - 2018.12.06 iOS12.1.1原生键盘内置小鹤双拼方案");
    p(ui, "    - 2018.12.06 macOS10.14.2原生键盘内置小鹤双拼方案");
    sp(ui);

    bul(ui, "未来可期：");
    p(
        ui,
        "    - 小鹤音形，才是小鹤方案的设计目标，更自由高效的输入体验",
    );
    p(
        ui,
        "    - 小鹤双拼已经有逐步成为小众中之大众的趋势，小鹤音形能随之受到重视算是未来愿景",
    );
    sp(ui);

    qt(ui, "作　者：何海峰（散步的鹤）");
    ui.horizontal(|ui| {
        label_hl(ui, "官　网：");
        ext_link(ui, "https://flypy.cc", "https://flypy.cc");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "Ｑ　群：");
        ext_link(ui, "182883808", "tencent://message/?uin=182883808");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "E_mail：");
        ext_link(ui, "flypy@qq.com", "mailto:flypy@qq.com");
    });
    sp(ui);

    qt(ui, "祝福远方的友人安好！");
    qt(ui, "南京的雨花台 珠海的太禾 深圳的蔚深");
    qt(ui, "小鹤双拼一直都在你们身边");

    hr(ui);
    navrow(ui, nav, Some(("5 指引", "vy")), None);
}
