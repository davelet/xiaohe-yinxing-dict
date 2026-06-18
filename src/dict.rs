use std::fmt;

/// 词典条目分类枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    YiJiJianMa,
    ErChongJianMa,
    SanMaTianKong,
    SiMaQuanMaZi,
    SiMaQuanMaCiZhiDing,
    SiMaQuanMaCi,
    KuaiFu,
    FuHao,
    BuShouBuJian,
    Emoji,
    WeiXinBiaoQing,
    WangZhanZhiDa,
    SuiXinSuoYu,
    ErJianCiXuan,
    SiMaCiXuan,
    ShouXuanZiCi,
}

impl Category {
    /// 获取分类的详细描述
    pub fn description(&self) -> &'static str {
        match self {
            Category::YiJiJianMa => {
                "一级简码是小鹤音形中最常用的26个汉字，每个字母对应一个高频字。\n\n例如：\n- q = 起\n- w = 我\n- e = 而\n- r = 人\n直接按一个字母加空格即可输入。"
            }
            Category::ErChongJianMa => {
                "二重简码是使用两个字母编码的常用汉字，约有600多个。\n\n例如：\n- qb = 情\n- qc = 请\n- qd = 巧\n输入两个字母后按空格即可输入。"
            }
            Category::SanMaTianKong => {
                "三码填空是指三码编码的汉字，在输入三码后系统会自动填空上屏。\n\n这是小鹤音形的特色功能，无需按空格确认，大大提升输入速度。"
            }
            Category::SiMaQuanMaZi => {
                "四码全码（单字）是完整的四码编码汉字。\n\n编码规则：声 + 韵 + 首形 + 尾形\n\n当输入四码时，如果只有一个候选字，会自动上屏。"
            }
            Category::SiMaQuanMaCiZhiDing => {
                "四码全码（词置顶）是指在词库中优先级最高的词组，会在候选框中置顶显示。\n\n这些词组通常是最常用的固定搭配。"
            }
            Category::SiMaQuanMaCi => {
                "四码全码（词）是普通词组，按照词组编码规则输入。\n\n双字词：首字前两码 + 次字前两码\n三字词：前两字首码 + 第三字前两码\n四字及以上词：前三字首码 + 末字首码"
            }
            Category::KuaiFu => {
                "快符是快速输入特殊符号的功能。\n\n例如：\n- ; = 。\n- ;; = ；\n- ;a = ！\n- ;b = （\n输入分号后跟一个字母即可快速输入对应符号。"
            }
            Category::FuHao => {
                "符号分类包含各种特殊符号和标点。\n\n包含：数学符号、标点符号、箭头符号、括号符号等。\n\n可以通过编码反查来快速找到需要的符号。"
            }
            Category::BuShouBuJian => {
                "部首部件是汉字的基本组成部分，了解这些部件有助于理解和记忆字形编码。\n\n每个部首都有对应的编码，掌握后可以更准确地拆分生僻字。"
            }
            Category::Emoji => {
                "Emoji表情符号，可以通过编码输入各种表情。\n\n例如：\n- hh = 😄 (哈哈)\n- kx = 😊 (开心)\n- wq = 😢 (委屈)\n通过拼音首字母即可快速输入常用表情。"
            }
            Category::WeiXinBiaoQing => {
                "微信表情是微信中常用的表情图。\n\n可以通过编码快速输入对应的微信表情文字描述或快捷短语。"
            }
            Category::WangZhanZhiDa => {
                "网站直达是通过编码快速打开常用网站的功能。\n\n例如输入对应编码后按指定键即可在浏览器中打开网站。"
            }
            Category::SuiXinSuoYu => {
                "随心所欲是一些自定义的特殊短语和快捷输入。\n\n包含各种实用的快捷短语和特殊功能，方便快速输入长文本。"
            }
            Category::ErJianCiXuan => {
                "二简（次选字）是二重简码的次选字，即两个字母编码的第二个候选字。\n\n通常按分号键选择次选，按引号键选择三选。"
            }
            Category::SiMaCiXuan => {
                "四码（次选词）是四码全码词组的次选候选词。\n\n当多个词组编码相同时，按分号可以选择次选词，按引号可以选择三选词。"
            }
            Category::ShouXuanZiCi => {
                "首选字词是四码编码时的第一候选，即最常用的字词。\n\n输入四码后直接按空格即可上屏，无需选择。"
            }
        }
    }

    /// 返回"全部"分类的描述
    pub fn all_description() -> &'static str {
        "全部分类显示词典中所有类型的条目。\n\n小鹤音形是一款音形码输入法，结合了拼音和字形的优点：\n\n• 音码：双拼方案，每个拼音两码完成\n• 形码：基于汉字首尾部结构\n\n特点：\n- 低重码：音形结合大幅降低重码\n- 盲打：低重码支持真正的盲打\n- 易学：双拼+简单字形规则\n- 高效：自动填空、四码唯一自动上屏\n\n建议从一级简码和二重简码开始学习，逐步掌握三码和四码全码。"
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Category::YiJiJianMa => "一级简码",
            Category::ErChongJianMa => "二重简码",
            Category::SanMaTianKong => "三码填空",
            Category::SiMaQuanMaZi => "四码全码（字）",
            Category::SiMaQuanMaCiZhiDing => "四码全码（词）置顶",
            Category::SiMaQuanMaCi => "四码全码（词）",
            Category::KuaiFu => "快符",
            Category::FuHao => "符号",
            Category::BuShouBuJian => "部首部件",
            Category::Emoji => "Emoji",
            Category::WeiXinBiaoQing => "微信表情",
            Category::WangZhanZhiDa => "网站直达",
            Category::SuiXinSuoYu => "随心所欲",
            Category::ErJianCiXuan => "二简（次选字）",
            Category::SiMaCiXuan => "四码（次选词）",
            Category::ShouXuanZiCi => "首选字词",
        }
    }

    pub fn all() -> Vec<Category> {
        vec![
            Category::YiJiJianMa,
            Category::ErChongJianMa,
            Category::SanMaTianKong,
            Category::SiMaQuanMaZi,
            Category::SiMaQuanMaCiZhiDing,
            Category::SiMaQuanMaCi,
            Category::ShouXuanZiCi,
            Category::KuaiFu,
            Category::FuHao,
            Category::BuShouBuJian,
            Category::Emoji,
            Category::WeiXinBiaoQing,
            Category::WangZhanZhiDa,
            Category::SuiXinSuoYu,
            Category::ErJianCiXuan,
            Category::SiMaCiXuan,
        ]
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// 词典条目
#[derive(Debug, Clone)]
pub struct DictEntry {
    pub text: &'static str,
    pub code: &'static str,
    pub category: Category,
    pub is_secondary: bool,
}

impl DictEntry {
    /// 获取所有存在的分类
    pub fn all_categories() -> Vec<Category> {
        Category::all()
    }
}
