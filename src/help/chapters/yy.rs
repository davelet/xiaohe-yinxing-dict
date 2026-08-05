use super::super::*;

pub(crate) fn render_yy(ui: &mut Ui, nav: &mut HelpNav, _manager: &HelpManager) {
    h2(ui, "2 输入法应用");

    h4(ui, "一、拼音内置——小鹤双拼");
    bul(ui, "小鹤官方发布");
    p(ui, "  「小鹤双拼」方案于2007年1月8日定稿并发布");
    bul(ui, "小鹤官方授权");
    p(ui, "  2010.03.25 搜狗拼音 5.0");
    p(ui, "  2010.05.07 QQ拼音 3.2");
    p(ui, "  2010.09.21 谷歌拼音 2.3.14.85");
    p(ui, "  2018.12.06 iOS12.1.1原生键盘");
    p(ui, "  2018.12.06 macOS10.14.2原生键盘");
    p(ui, "  ...");
    red(ui, "= 小鹤双拼用户可在设置中选择本方案使用 =");
    sp(ui);

    h4(ui, "二、独立应用——小鹤音形");
    bul(ui, "小鹤官方发布");
    p(ui, "  鹤形方案：2007年5月6日初稿 - 2008年2月23日定稿");
    p(
        ui,
        "  1.「小鹤音形」输入法Windows版，基于小鹤音形方案与多多输入法生成器生成",
    );
    p(
        ui,
        "  2.「小鹤音形」输入法Android版，基于小鹤音形方案与小胖输入法平台生成",
    );
    bul(ui, "小鹤官方授权");
    blue(
        ui,
        "小鹤音形尚无授权发布版本，任何第三方内置小鹤音形方案的行为均为侵权",
    );
    bul(ui, "挂接第三方");
    p(
        ui,
        "  Windows、iOS、macOS、Linux等各系统亦可通过挂接基于Rime框架的各系统前端（软件平台）实现，或其他平台实现",
    );
    p(ui, "  挂接文件在小鹤官方网盘中提供，见三");
    red(ui, "　　= 小鹤音形用户可安装或挂接使用 =");
    sp(ui);

    h4(ui, "三、下载地址");
    bul(ui, "小鹤音形下载：");
    ui.horizontal(|ui| {
        label_hl(ui, "  官　网：");
        ext_link(ui, "https://flypy.cc", "https://flypy.cc");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  网　盘：");
        ext_link(ui, "http://flypy.ysepan.com/", "http://flypy.ysepan.com/");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  QQ群：");
        ext_link(ui, "182883808", "tencent://message/?uin=182883808");
    });
    bul(ui, "小鹤双拼下载：");
    p(
        ui,
        "  请到各大拼音输入法官方网站下载其拼音输入法，在其设置中选择小鹤双拼使用",
    );

    hr(ui);
    navrow(ui, nav, _manager, Some("zg"), Some("jm"));
}
