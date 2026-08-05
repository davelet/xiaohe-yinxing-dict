use super::super::*;

pub(crate) fn render_pc(ui: &mut Ui, nav: &mut HelpNav, _manager: &HelpManager) {
    h2(ui, "2.3 Win版 指南");
    blue(ui, "「小鹤音形」方案 ＋「多多输入法」平台");
    sp(ui);

    h4(ui, "一、安装");
    p(ui, "应用名称： FlyPY_Setup.exe");
    p(ui, "安装目录： C:\\Program Files\\FlyPYInput");
    sp(ui);

    h4(ui, "二、快符、O符");
    num(ui, "1", "快符： ");
    ui.horizontal(|ui| {
        code(ui, "分号键");
        label_hl(ui, " + ");
        code(ui, "任一字母");
        label_hl(ui, " 两键上屏的符号或执行的功能；双击分号上屏本身");
    });
    ui.horizontal(|ui| {
        code(ui, ";a");
        label_hl(ui, " 输出感叹号！（其它符号输出类推）");
    });
    ui.horizontal(|ui| {
        code(ui, ";i");
        label_hl(ui, " 撤销上屏词条");
    });
    ui.horizontal(|ui| {
        code(ui, ";f");
        label_hl(ui, " 重复上屏词条");
    });
    ui.horizontal(|ui| {
        code(ui, ";n");
        label_hl(ui, " 模拟 ");
        code(ui, "End");
        label_hl(ui, " 键，可用于光标移出成对符号");
    });
    qt(ui, "① * 号表示上屏内容后再使用此快符");
    qt(ui, "② 如不需要快符功能，可用直通 oyd 关闭");
    num(ui, "2", "O 符：以字母 ");
    ui.horizontal(|ui| {
        code(ui, "o");
        label_hl(ui, " 开始编码的符号数字等其它符号编码");
    });
    ui.horizontal(|ui| {
        code(ui, "ob");
        label_hl(ui, " 引导部件字根所在键列表，如：");
        code(ui, "obc");
        label_hl(ui, " 1.艹　2.廾");
    });
    ui.horizontal(|ui| {
        code(ui, "ox");
        label_hl(ui, " 引导小字字根所在键列表，如：");
        code(ui, "oxc");
        label_hl(ui, " 1.寸　2.才　3.册　4.匆");
    });
    ui.horizontal(|ui| {
        code(ui, "of");
        label_hl(ui, " 引导成组符号（编码见二）");
    });
    ui.horizontal(|ui| {
        code(ui, "ot");
        label_hl(ui, " 引导特殊符号（编码见三）");
    });
    ui.horizontal(|ui| {
        code(ui, "ow");
        label_hl(ui, " 引导微信表情，如：");
        code(ui, "owwx");
        label_hl(ui, " [微笑]");
    });
    ui.horizontal(|ui| {
        code(ui, "oi");
        label_hl(ui, " 引导emoji表情，如：");
        code(ui, "oixk");
        label_hl(ui, " 😂");
    });
    ui.horizontal(|ui| {
        code(ui, "op");
        label_hl(ui, " 引导拼音字母，如：");
        code(ui, "opa");
        label_hl(ui, " 1.ā　2.á");
    });
    ui.horizontal(|ui| {
        code(ui, "oe");
        label_hl(ui, " 引导音标字母，如：");
        code(ui, "oea");
        label_hl(ui, " 1.æ　2.ʌ　3.ɑ:");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "详见：");
        lnk(ui, nav, "符号", "fh");
    });
    sp(ui);

    h4(ui, "三、常用组合直通");
    tbl(
        ui,
        110.0,
        &["功能", "功能键", "功能", "组合键", "直通码"],
        &[
            &["中/英", "Shift", "全/半角", "（屏蔽）", "oqb"],
            &["次选上屏", ";", "中/英标点", "Ctrl + .", "ovy"],
            &["翻页", "[ ]", "简/繁", "Ctrl + Alt + f", "ojf"],
            &["快符引导", ";", "全码字", "Ctrl + Alt + q", "oqm"],
            &["便捷引导", "'", "二简次选", "Ctrl + Alt + j", "oej"],
            &["编码清屏", "Tab", "在线加词", "Ctrl + Alt + =", "ojc"],
            &["编码上屏", "Enter", "小鹤入门", "打开手册", "xhrm"],
        ],
    );
    sp(ui);

    h4(ui, "四、词库分类");
    num(ui, "1", "词库分类");
    ui.horizontal(|ui| {
        label_hl(ui, "输入");
        code(ui, "omb");
        label_hl(ui, "打开码表分类项，分类如下：");
    });
    tbl(
        ui,
        90.0,
        &["码表分类", "内容说明"],
        &[
            &["首选字词", "第一候选字词库"],
            &["次选字词", "第二候选字词库"],
            &[
                "表外字",
                "《通用规范汉字表》国发〔2013〕23号文规定用字之外的字",
            ],
            &[
                "随心",
                "自定构词规则或特殊分类，零星调节系统词条重码或别扭码",
            ],
            &["直通", "特殊词条，开关启动等功能"],
            &["快符", "快符：分号加字母两键上屏标点"],
            &[
                "Ｏ符",
                "o引导符号，包含各种数字字母货币制表等符号及鹤形字根、QQ表情等",
            ],
            &["全码词", "部分简码词补全码"],
            &["全码字", "部分已有简码的全码字"],
            &["拼字", "ok引导拼字输入表外字"],
            &[
                "用户",
                "ojc添加的词归入此分类，亦可从离线词库导入，含一简次选",
            ],
            &["二简次选", "二简第二候选字，默认关闭，oej启用"],
        ],
    );
    qt(ui, "鼠标右键点击各分类，进行“编辑、导入、导出、清空”等操作");
    num(ui, "2", "用户词库");
    p(
        ui,
        "用户词库可以直接用直通 ojc 方式加词，也可从外部离线 txt 文件导入方式加词",
    );
    sp(ui);

    h4(ui, "五、查码查形");
    num(ui, "1", "查码：不知道“编码”时的查询方法");
    bul(
        ui,
        "知形查音码：不知道读音可先输入两位查询字符 分别替代双拼两码，再继续输入双形两码，从候选字中查看双拼编码",
    );
    bul(
        ui,
        "知音查形码：不知道双形可先输入双拼两码，再输入两位查询字符 分别替代双形两码，从候选字中查看双形编码",
    );
    qt(
        ui,
        "查询字符（又称万能键）， 此符号键位于 Tab 键上方，替代任一编码",
    );
    bul(ui, "复制反查：将要查的字复制后，输入 ofi 查询编码");
    num(ui, "2", "查形：不知道“字根”时的查询方法");
    bul(ui, "方法①：字+ ozd （打开自带字典查形）");
    bul(ui, "方法②：字+ oix （打开网页查形）");
    sp(ui);

    h4(ui, "六、便捷输入");
    p(
        ui,
        "可快速输入英文、日期、数字、金额或临时启用未开启分类、英文输入等",
    );
    ui.horizontal_wrapped(|ui| {
        label_hl_mk(ui, "单引号", |s| RichText::new(s).strong());
        label_hl(ui, "：引导便捷输入，双击上屏本身");
    });
    bul(ui, "任意日期");
    ui.horizontal(|ui| {
        label_hl(ui, "  输入");
        code(ui, "'2007.1.8");
        label_hl(ui, "　候选：a. 二〇〇七年一月八日　b. 2007年1月8日");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  输入");
        code(ui, "'2008.2.");
        label_hl(ui, "　候选：a. 二〇〇八年二月　b. 2008年2月");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  当前日期、时间可使用直通码方式：");
        code(ui, "orq");
        code(ui, "ouj");
    });
    bul(ui, "任意金额");
    ui.horizontal(|ui| {
        label_hl(ui, "  输入");
        code(ui, "'2019.12");
        label_hl(
            ui,
            "　候选：a. 二千零一十九点一二　b. 贰仟零壹拾玖元壹角贰分",
        );
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  输入");
        code(ui, "'2019");
        label_hl(ui, "　候选：a. 二千零一十九　b. 贰仟零壹拾玖元整");
    });
    bul(ui, "临时生僻");
    ui.horizontal(|ui| {
        label_hl(ui, "  临时显示＜全码字＞（含生僻字）分类，如：");
        code(ui, "'bulw");
        label_hl(ui, "　瓿");
    });
    ui.horizontal_wrapped(|ui| {
        label_hl(ui, "  也可用组合键");
        code(ui, "Ctrl");
        code(ui, "Alt");
        code(ui, "q");
        label_hl(ui, "或输入 ");
        code(ui, "oqm");
        label_hl(ui, " 显示分类直接输入");
    });
    bul(ui, "临时二简次选");
    ui.horizontal(|ui| {
        label_hl(ui, "  临时启用＜二简次选＞分类，如：");
        code(ui, "'bw");
        label_hl(ui, "　背");
    });
    ui.horizontal_wrapped(|ui| {
        label_hl(ui, "  也可用组合键");
        code(ui, "Ctrl");
        code(ui, "Alt");
        code(ui, "j");
        label_hl(ui, "或直通车 ");
        code(ui, "oej");
        label_hl(ui, " 启用分类直接输入");
    });
    bul(ui, "临时英文");
    p(ui, "  临时启用英文模式，完成后用引导键结束，并上屏英文");
    sp(ui);

    h4(ui, "七、智能标点");
    bul(ui, "可快速把中文标点转换为英文标点");
    p(ui, "如：双击逗号变英文逗号（半秒内），超时则不变");
    bul(
        ui,
        "高级设置→“智能标点表”，等号后所列标点（英文）即为可转换标点",
    );
    sp(ui);

    h4(ui, "八、OK拼字");
    bul(
        ui,
        "支持GB 18030-2022，可用于《通用规范汉字表》外的文字输入",
    );
    p(
        ui,
        "使用 ok+二分双拼码 方式进行输入，二分不能拼完的字，继续三分",
    );
    p(ui, "如：okhoho 炎，okhohoho 焱");
    qt(ui, "辶廴 归到 vi，礻衤归到 pp");
    sp(ui);

    h4(ui, "九、备选分类词库：二简次选");
    bul(ui, "二简字有一套第二候选分类词库供选择：");
    num(ui, "1", "默认 = 主码表");
    ui.horizontal(|ui| {
        label_hl(ui, "  ");
        label_hl_mk(ui, "bw　1.被", |s| RichText::new(s).strong());
    });
    num(ui, "2", "备选 = 主码表＋");
    ui.horizontal(|ui| {
        label_hl(ui, "  ");
        label_hl_mk(ui, "bw　1.被　2.背", |s| RichText::new(s).strong());
    });
    qt(ui, "oej 启用 <二简次选> 分类");
    sp(ui);

    h4(ui, "十、词库进阶：初学 → 熟手");
    bul(ui, "词库的使用分成三个阶段：初学 → 常规 → 熟手");
    num(
        ui,
        "1",
        "初学阶段：显示 ＜全码字、词＞ 分类，初学者学习单字全码拆分用到",
    );
    num(
        ui,
        "2",
        "常规阶段：隐藏 ＜全码字> 分类 oqm ，实际使用阶段，单字有简打简",
    );
    num(
        ui,
        "3",
        "熟手阶段：隐藏 <全码字> 分类，清空 <全码词> 分类，熟悉部分高频二简词",
    );
    bul(
        ui,
        "初学到熟手，就是对词库做减法：1 - ＜全码字＞ = 2 - ＜全码词＞ = 3",
    );
    p(ui, "即，分类词库启用情况如下：");
    p(ui, "初学词库：<系统>");
    p(ui, "常规词库：<系统> - <全码字>");
    p(ui, "熟手词库：<系统> - <全码字> - <全码词>");
    qt(ui, "默认：初学阶段");
    qt(ui, "＜全码字＞部分已出简码的字的全码&生僻字");
    qt(ui, "＜全码词＞部分已出简码的词的全码");

    hr(ui);
    navrow(
        ui,
        nav,
        _manager,
        Some("fh"),
        Some("sj"),
    );
}
