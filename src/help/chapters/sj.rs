use super::super::*;

pub(crate) fn render_sj(ui: &mut Ui, nav: &mut HelpNav, _manager: &HelpManager) {
    h2(ui, "2.4 安卓版 指南");
    blue(ui, "「小鹤音形」方案 ＋「小胖输入法」平台");
    sp(ui);

    h4(ui, "一、安装");
    p(ui, "应用名称：FlyPYime.apk");
    p(
        ui,
        "安装目录：/storage/emulated/0/Android/data/cc.flypy.input/",
    );
    qt(ui, "下文中：");
    qt(
        ui,
        "1. 字母后有背景阴影的都表示在中文模式下的敲击键位，如：oei",
    );
    qt(ui, "2. 无背景阴影则表示英文字母本身");
    qt(ui, "3. _ 表示空格");
    qt(ui, "4. ⤴ 表示引导键，键盘上有此标记的键");
    qt(
        ui,
        "推荐安装文件管理编辑应用“MT管理器”，文件夹及文件的直达及管理编辑将使用到",
    );
    ui.horizontal(|ui| {
        label_hl(ui, "MT管理器下载地址：");
        ext_link(ui, "https://mt2.cn/download/", "https://mt2.cn/download/");
    });
    sp(ui);

    h4(ui, "二、符号及键盘功能图示");
    num(ui, "1", "键盘功能");
    ui.horizontal(|ui| {
        label_hl(ui, "  - ");
        code(ui, "O");
        label_hl(ui, " 键引导日常符号，可参看 ");
        lnk(ui, nav, "2.2 符号", "fh");
        label_hl(ui, " 篇");
    });
    bul(ui, "虚拟键盘：");
    ui.horizontal(|ui| {
        label_hl(ui, "    1. ");
        code(ui, "⤴");
        code(ui, "逗号");
        code(ui, "字母");
        label_hl(ui, "构成 快直通");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "    2. ");
        code(ui, "⤴");
        code(ui, "⤴");
        code(ui, "字母");
        label_hl(ui, "构成 快符");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "    3. ");
        code(ui, "⤴");
        code(ui, "字母数字");
        label_hl(ui, "引导 符号组、计算、生僻字、英文");
    });
    bul(ui, "外接键盘：");
    ui.horizontal(|ui| {
        label_hl(ui, "    1. ");
        code(ui, ";");
        code(ui, "字母");
        label_hl(ui, "构成 快符 ， ");
        code(ui, ";b");
        label_hl(ui, " 为 逆切分，其他快符见 ");
        lnk(ui, nav, "2.3 win版", "pc");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "    2. ");
        code(ui, "单引号");
        code(ui, "字母数字");
        label_hl(ui, "引导 符号组、计算、生僻字、英文");
    });
    num(ui, "2", "上中排下滑符号");
    bul(ui, "可通过用户词库编码方式进行自定义");
    sp(ui);

    h4(ui, "三、键盘相关");
    label_hl_mk(ui, "主键盘功能：", |s| RichText::new(s).strong());
    num(ui, "1", "点击功能：");
    ui.horizontal(|ui| {
        label_hl(ui, "  a. ");
        code(ui, "shift");
        label_hl(ui, " 切换大写，有候选时清码");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  b. ");
        code(ui, "ϟ12");
        label_hl(ui, " 切换到“数字和符号键盘”");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  c. ");
        code(ui, "句号");
        label_hl(ui, " 有候选时做次选键");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  d. ");
        code(ui, "逗号");
        label_hl(ui, " 有候选时做三选键");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  e. ");
        code(ui, "···");
        label_hl(ui, " 切换输入法，有候选时为句号");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  f. ");
        label_hl_mk(ui, "空格下方", |s| RichText::new(s).strong());
        label_hl(ui, "是 ");
        code(ui, "左右方向");
        label_hl(ui, " 键，有候选时做空格");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  g. ");
        code(ui, "☰");
        label_hl(ui, " 进入“功能键盘”");
    });
    num(ui, "2", "上滑和长按功能：");
    ui.horizontal(|ui| {
        label_hl(ui, "  a. ");
        code(ui, "shift");
        label_hl(ui, " 上滑为开关状态栏，长按切换“日夜皮肤”");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  b. ");
        code(ui, "ϟ12");
        label_hl(ui, " 上滑打开“编辑键盘”，长按进入“功能键盘”");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  c. ");
        code(ui, "空格");
        label_hl(ui, " 上滑展开候选（大于2时），长按切换中英文键盘");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  d. ");
        code(ui, "⌫");
        label_hl(ui, " 上滑撤销上屏或纠错，长按连续删");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  e. ");
        code(ui, "回车");
        label_hl(ui, " 上滑重复上屏或诗词补全，长按换行");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  f. ");
        label_hl_mk(ui, "键上档", |s| RichText::new(s).strong());
        label_hl(ui, "标点及功能，通过上滑或长按作用");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  g. ");
        label_hl_mk(ui, "全选、剪切、复制、粘贴", |s| {
            RichText::new(s).strong()
        });
        label_hl(ui, " 分别放在 ");
        code(ui, "AXCV");
        label_hl(ui, " 键上档");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  h. ");
        code(ui, "Z");
        label_hl(ui, " 键  上滑长按均为 ");
        code(ui, "万能键");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  i. ");
        code(ui, "☰");
        label_hl(ui, " 长按弹出“键盘”选单");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  j. ");
        code(ui, "﹀");
        label_hl(ui, " 长按弹出“皮肤”选单");
    });
    num(ui, "3", "下滑功能：");
    ui.horizontal(|ui| {
        label_hl(ui, "  a. ");
        code(ui, "Z");
        label_hl(ui, "键出 ");
        code(ui, "TAB");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  b. ");
        code(ui, "shift");
        label_hl(ui, " 选行");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  c. ");
        code(ui, "⌫");
        label_hl(ui, " 关窗+删行");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  d. ");
        code(ui, "回车");
        label_hl(ui, " 恢复删行");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  e. ");
        code(ui, "空格");
        label_hl(ui, " 逆切分");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  f. ");
        code(ui, "XCVBN");
        label_hl(ui, " 分别跳转“剪中英表特”辅键盘");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  g. ");
        code(ui, "逗号");
        label_hl(ui, " 英文键盘为英文补全开关");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  h. 下滑可选择“下滑助记”键盘查看");
    });
    num(ui, "4", "滑动功能：");
    ui.horizontal(|ui| {
        label_hl(ui, "  a. ");
        code(ui, "⌫");
        label_hl(
            ui,
            " 开始左滑，删除前面的内容，左滑的位置继续右滑则恢复删除的内容",
        );
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  b. ");
        code(ui, "空格");
        label_hl(ui, " 左右两端开始左右滑动，光标左右移动");
    });
    num(ui, "5", "状态标志显示含义：");
    ui.horizontal(|ui| {
        label_hl(ui, "  a. ");
        code(ui, "☰");
        label_hl(ui, " 在切换到繁体时显示为 ");
        code(ui, "☷");
    });
    ui.horizontal(|ui| {
        label_hl(
            ui,
            "  b. 状态栏右侧隐藏键，在有更多候选时变换图标表示可展开",
        );
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  c. ");
        code(ui, "空格");
        label_hl(
            ui,
            " 上的文字：+小鹤　小鹤　-小鹤，分别表示词库的三种模式：初学　常规　熟手",
        );
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  d. ");
        code(ui, "空格");
        label_hl(
            ui,
            " 文本 “—” 表达半角状态，“—” 表达全角状态，“⌒”表达英文补全状态",
        );
    });
    p(
        ui,
        "　　　　图片“SPACE”表达半角状态，“S P C”表达全角状态，“ENGLISH”表达英文补全状态",
    );
    num(ui, "6", "候选窗操作：");
    p(
        ui,
        "  a. 手动调频（默认关闭），长按候选项置顶，记录在sys-reset.txt文件中",
    );
    sp(ui);

    label_hl_mk(ui, "辅键盘功能：", |s| RichText::new(s).strong());
    num(ui, "1", "数字键盘等号后可输入金额、日期或进行简单计算");
    num(
        ui,
        "2",
        "符号键盘点选后上屏符号并跳转主键盘，滑选或长按则只上屏符号不跳转键盘",
    );
    sp(ui);

    h4(ui, "四、查码&查形");
    num(
        ui,
        "1",
        "查码：上滑或长按 z 键是万能键，用于查码（外接键盘时是 ` 符号，Tab 上方，替代任一码）",
    );
    bul(ui, "知形查音码");
    p(
        ui,
        "  在不知道读音的情况下，用万能键分别代替双拼两码，再输入双形两码，来反查双拼的编码，如：ji 反查“钏”字的读音",
    );
    bul(ui, "知音查形码");
    p(
        ui,
        "  在不知道形编码的情况下，先输入双拼两码，再用万能键分别代替双形两码，来反查双形的编码，如：ji 反查“瘠”字的形编码",
    );
    bul(ui, "剪贴板反查编码");
    p(ui, "  复制某个字后，使用直通车 ofi 查询字的编码");
    num(
        ui,
        "2",
        "查形：用于知道形的编码，但不知道代表哪个字根的情况",
    );
    bul(ui, "本地");
    p(ui, "   方法：① 字+ohh  ② 字+⤴+ϟ12");
    p(ui, "   如： 瘠 ohh 　　　结果得到 → 瘠：疒 月　jibo");
    bul(ui, "网页");
    p(ui, "   方法：字+oix");
    p(
        ui,
        "   如：羲 oix 　　　直通跳转网页查光标前字，无字则取剪贴板",
    );
    qt(ui, "万能键查询，候选标志含义：");
    qt(
        ui,
        "1. - 表示有简码全码让出首选位， oqm 可切换隐藏或居后模式",
    );
    qt(ui, "2. * 表示生僻字（音）， oqm 可切换隐藏或居后模式");
    qt(ui, "3. + 表示《通用规范汉字表》外收录的字");
    sp(ui);

    h4(ui, "五、词库使用");
    num(
        ui,
        "1",
        "不同于其他输入法，多了一个 暗词库 的分类，传统的词库在这里我们称为 明词库",
    );
    bul(ui, "明词库：输入编码则输出编码对应的词条");
    bul(ui, "暗词库：根据前缀词条输出的后缀编码词条");
    p(
        ui,
        "    暗词库有点类似拼音里的上下文调频，比如上文上屏了“工作”，后面接着打 vg 首候选就可能出现的是“证”，而没有上文的“工作”时，打 vg 可能首候选是“正”。",
    );
    label_hl_mk(ui, "  暗词库的优势：", |s| RichText::new(s).strong());
    qt(ui, "① 可控，这应该是最关键的，暗词库完全由自己控制");
    qt(ui, "② 可替换前缀词条，达到纠错或其他功能的目的");
    qt(ui, "③ 结合转码直通，可实现一些特殊功能");
    num(ui, "2", "二简词");
    p(
        ui,
        "   这类词没有单独做分类码表，并入主码表，在本手册“2.1 简码”篇有列表",
    );
    p(ui, "   本类词条提供了助记词库，通过 oejc 开启");
    num(ui, "3", "二简次选");
    qt(ui, "① 默认＝主码表");
    ui.horizontal(|ui| {
        qt(ui, "bw　1.被");
    });
    qt(ui, "② 备选＝主码表＋＜二简次选＞");
    ui.horizontal(|ui| {
        qt(ui, "bw　1.被　2.背");
    });
    qt(ui, "oej 启用 <二简次选> 分类");
    num(ui, "4", "用户词库");
    p(ui, "   内词库：小鹤音形/2.3.用户词库.txt");
    p(
        ui,
        "   外词库：$userpath$/小鹤用户词库.txt （需自建，在高级设置界面设置外词库所在目录，$userpath$ 为变量，表示所选目录）",
    );
    qt(ui, "oyh 直通用于打开内外词库");
    num(ui, "5", "排序调频");
    bul(ui, "词库排序调频");
    qt(ui, "① 在词条尾部+ #固 方式置顶重码的用户词");
    qt(ui, "② 在词条尾部+ #末 方式置末重码的用户词");
    qt(ui, "③ 在词条尾部+ #2 或 #3 方式置用户词于23候选");
    qt(ui, "④ 在词条尾部+ #删 删除已有词，从而让新加的同码词置顶");
    bul(ui, "候选窗排序调频（默认关闭）");
    p(ui, "通过长按候选置顶");
    qt(ui, "候选窗调频记录在 sys-reset.txt 文件中");
    qt(ui, "opx 可开关候选窗调频功能");
    sp(ui);

    h4(ui, "六、直通码");
    num(ui, "1", "“2.3.直通-安卓.txt”");
    ui.horizontal(|ui| {
        label_hl(ui, "可通过直通编码 ");
        code(ui, "ovt");
        label_hl(ui, " 打开直通文件");
    });
    bul(ui, "常用直通码：");
    tbl(
        ui,
        90.0,
        &["直通功能", "直通码", "直通功能", "直通码"],
        &[
            &["重载", "oiz", "用户", "oyh"],
            &["设置", "ocd", "键高", "ojg"],
            &["日期", "orq", "字典", "ozd"],
            &["时间", "ouj", "简繁", "ojf"],
            &["候选", "ohx", "静音", "ojy"],
            &["进阶", "ojj", "无刻", "owk"],
            &["输入模式", "oit", "全码字", "oqm"],
            &["二简次选", "oej", "使用入门", "orm"],
        ],
    );
    qt(ui, "更多直通编码见“2.3.直通.txt”词库， ovt 打开");
    num(ui, "2", "转码直通");
    p(ui, "   直通命令有很多参数，这里专门讲下转码直通的应用");
    bul(ui, "直通词条格式： $cmd(命令字符串,命令说明)+TAB符+编码");
    p(ui, "   转码命令字符串：newkey(...)");
    p(ui, "   例： $cmd(newkey(ovxh$1),J双拼)\\t,h");
    num(ui, "3", "键盘有关直通");
    bul(ui, "键盘字体");
    p(
        ui,
        "   键盘字体包括：按键字体、候选字体、角标字体，均可自定",
    );
    p(
        ui,
        "   自定字体名称：ziti.ttf，放到 ock 打开的目录下，通过ozt 切换自定字体和系统字体",
    );
    p(ui, "   字体粗细： ozt 3 切换（系统字体有效）");
    p(ui, "   字体大小：");
    ui.horizontal(|ui| {
        label_hl(ui, "     ");
        code(ui, "ohz");
        label_hl(ui, " 固定候选字号");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "     ");
        code(ui, "ofz");
        label_hl(ui, " 浮动候选字号");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "     ");
        code(ui, "ofzi");
        label_hl(ui, " 外接浮动候选字号");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "     ");
        code(ui, "ojz");
        label_hl(ui, " 键盘字号");
    });
    bul(ui, "键盘高度");
    ui.horizontal(|ui| {
        label_hl(ui, "     ");
        code(ui, "ojg");
        label_hl(ui, " 键盘高度");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "     ");
        code(ui, "odg");
        label_hl(ui, " 架空高度");
    });
    bul(ui, "横屏键盘");
    ui.horizontal(|ui| {
        label_hl(ui, "     ");
        code(ui, "ohp");
        label_hl(ui, " 横屏样式");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "     ");
        code(ui, "otmd");
        label_hl(ui, " 横屏透明度");
    });
    sp(ui);

    h4(ui, "七、数字键盘");
    p(
        ui,
        "除了数字键盘功能， = 引导时可用做“简易计算、任意金额、任意日期”",
    );
    p(ui, "通常是金额，加减乘除后进入计算式");
    sp(ui);

    h4(ui, "八、打简出繁");
    num(ui, "1", "简繁输入切换开关： ojf");
    num(ui, "2", "临时转繁体");
    p(ui, "在简体状态也可以临时转繁体");
    p(ui, "格式：字+ of");
    sp(ui);

    h4(ui, "九、剪贴板");
    bul(ui, "打开方法： 下滑 X 键");
    bul(ui, "特别用法：");
    ui.horizontal(|ui| {
        label_hl(ui, "  ");
        code(ui, "⤴，v");
        label_hl(ui, " 粘贴剪贴板第二条");
    });
    ui.horizontal(|ui| {
        label_hl(ui, "  ");
        code(ui, "⤴，b");
        label_hl(ui, " 粘贴剪贴板第三条");
    });
    qt(ui, "默认：保留一天  30条");
    qt(ui, "ojt 1打开剪贴板 2保留一天 3保留七天");
    qt(ui, "ojtb： 30条 60条 100条");
    qt(ui, "长按锁定，左滑删除，右滑打散");
    sp(ui);

    h4(ui, "十、智能标点");
    bul(ui, "中文标点转英文标点");
    p(
        ui,
        "如：双击逗号变英文逗号（半秒内），超时则不变，三击恢复逗号",
    );
    ui.horizontal(|ui| {
        code(ui, "ovn");
        label_hl(ui, " 可关闭此功能");
    });
    bul(ui, "数字后标点");
    p(ui, "如：数字后中文句号变英文句点");
    ui.horizontal(|ui| {
        code(ui, "osz");
        label_hl(ui, " 可关闭此功能");
    });
    sp(ui);

    h4(ui, "十一、OK 拼字");
    p(
        ui,
        "支持GB 18030-2022，可用于《通用规范汉字表》外的文字输入",
    );
    ui.horizontal(|ui| {
        label_hl(ui, "使用 ");
        code(ui, "ok");
        label_hl(ui, "+ ");
        code(ui, "二分双拼码");
        label_hl(ui, " 方式进行输入，二分不能拼完的字，继续三分");
    });
    p(ui, "如： okhoho 炎， okhohoho 焱");
    qt(ui, "辶廴 归到 vi，礻衤归到 pp");
    sp(ui);

    h4(ui, "十二、AI & 语音");
    num(ui, "1", "注册");
    ui.horizontal_wrapped(|ui| {
        label_hl(ui, "AI注册DeepSeek用户：");
        ext_link(ui, "https://deepseek.com/", "https://deepseek.com/");
    });
    p(ui, "创建API key");
    ui.horizontal_wrapped(|ui| {
        label_hl(ui, "语音注册豆包火山引擎：");
        ext_link(
            ui,
            "https://www.volcengine.com/",
            "https://www.volcengine.com/",
        );
    });
    num(ui, "2", "AI使用");
    qt(
        ui,
        "① 把DeepSeek用户中创建的API key值 填入直通词条的对应位置",
    );
    qt(ui, "② 重载 oiz");
    qt(ui, "③ 使用 olai 把key值写入配置文件并启用");
    qt(ui, "④ 输入你想要AI回复的内容，使用 oai 得到回复");
    num(ui, "3", "语音使用");
    qt(ui, "① 把参数填入直通词条的对应位置");
    qt(ui, "② 重载 oiz");
    qt(ui, "③ 使用 olyy 把参数写入配置文件并启用");
    qt(ui, "④ 开通麦克风权限");
    qt(ui, "⑤ 长按高飞键盘架空行的最右边按钮，震动后说话就好");
    qt(ui, "⑥ 外接键盘时，默认按住 右Alt 说话");
    sp(ui);

    h4(ui, "十三、备份 & 同步");
    bul(ui, "手动备份 & 同步");
    num(
        ui,
        "1",
        "直通 ock 打开词库目录，直接复制“2.3.用户词库.txt”到一个安全的地方",
    );
    num(
        ui,
        "2",
        "通过第三方app，如foldersync，同步手机端文件和坚果云端文件",
    );
    num(ui, "3", "外词库本身在安装目录外，相对安全");
    bul(ui, "自动备份 & 同步");
    num(
        ui,
        "1",
        "在“高级设置-设置用户词库”项选定了目录，会自动备份到此目录",
    );
    num(
        ui,
        "2",
        "自带webdav方式同步文件，可配合坚果云等进行自用词库、皮肤的同步",
    );
    qt(ui, "同步： otbu　1.同步　2.上传　3.下载");
    sp(ui);

    h4(ui, "十四、词库进阶");
    bul(ui, "词库的使用分成三个阶段：初学 → 常规 → 熟手");
    num(
        ui,
        "1",
        "初学阶段，显示 [全码字、词+生僻字] 分类，空格标志：＋小鹤",
    );
    num(ui, "2", "常规阶段，隐藏分类，空格标志：　小鹤");
    num(
        ui,
        "3",
        "熟手阶段，隐藏全码字、词+生僻字分类，空格标志：－小鹤",
    );
    bul(
        ui,
        "初学到熟手，就是对词库做减法：1 - ＜全码字、生僻字＞ = 2 - ＜全码词＞ = 3",
    );
    qt(ui, "三个阶段可通过直通码 ojj 切换");
    sp(ui);

    h4(ui, "十五、输入模式");
    num(ui, "1", "传统输入模式：顺切分模式");
    bul(ui, "自动切分：打完四码自动切断与后面编码的关系");
    bul(ui, "手动切分：用空格或标点打断与后面编码的关系");
    num(ui, "2", "切分输入模式：逆切分模式");
    bul(
        ui,
        "自动切分：打完四码如果是空码，则自动切分为2+2，即两个二简字词",
    );
    bul(ui, "手动切分：打完编码用切分键按规则重新切分编码");
    p(ui, "操作过程：编码+下滑空格");
    qt(ui, "两种模式通过 oit 切换");
    qt(ui, "模式2兼容1");
    qt(ui, "模式2需熟悉二简字词");
    qt(ui, "初学者使用半年后再考虑2模式");
    sp(ui);

    h4(ui, "十六、表情输入");
    num(ui, "1", "键盘表情");
    p(ui, "   手机输入法常见的表情输入方式：点开表情键盘选择表情");
    num(ui, "2", "编码表情");
    p(ui, "   小鹤常用的表情输入方式：");
    p(ui, "  在“2.6.符号.txt”文件内");
    ui.horizontal(|ui| {
        code(ui, "oq");
        label_hl(ui, "引导 QQ 表情");
    });
    ui.horizontal(|ui| {
        code(ui, "ow");
        label_hl(ui, "引导 微信 表情");
    });
    ui.horizontal(|ui| {
        code(ui, "oi");
        label_hl(ui, "引导 emoji 表情");
    });
    num(ui, "3", "emoji表情列表");
    ui.horizontal(|ui| {
        code(ui, "⤴，m");
        label_hl(ui, " 方式直接打开表情列表选择输入");
    });

    hr(ui);
    navrow(ui, nav, _manager, Some("pc"), Some("gj"));
}
