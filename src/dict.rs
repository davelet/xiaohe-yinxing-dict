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
    ShouXuanSiMa,
}

impl Category {
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
            Category::ShouXuanSiMa => "首选四码（词/短语）",
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
            Category::KuaiFu,
            Category::FuHao,
            Category::BuShouBuJian,
            Category::Emoji,
            Category::WeiXinBiaoQing,
            Category::WangZhanZhiDa,
            Category::SuiXinSuoYu,
            Category::ErJianCiXuan,
            Category::SiMaCiXuan,
            Category::ShouXuanSiMa,
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
