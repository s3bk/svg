use isolang::Language;
use font::opentype::Tag;

fn tag(a: &[u8; 4]) -> Option<Tag> {
    if a != &[0, 0, 0, 0] {
        Some(Tag(*a))
    } else {
        None
    }
}

pub fn lang_to_tag(lang: Language) -> Option<Tag> {
    use Language::*;
    match lang {
        // Esperanto (Esperanto)
        Epo => tag(b"NTO "),

        // English (English)
        Eng => tag(b"ENG "),

        // Русский (Russian)
        Rus => tag(b"RUS "),

        // 普通话 (Mandarin)
        Cmn => None,

        // Español (Spanish)
        Spa => tag(b"ESP "),

        // Português (Portuguese)
        Por => tag(b"PTG "),

        // Italiano (Italian)
        Ita => tag(b"ITA "),

        // বাংলা (Bengali)
        Ben => tag(b"BEN "),

        // Français (French)
        Fra => tag(b"FRA "),

        // Deutsch (German)
        Deu => tag(b"DEU "),

        // Українська (Ukrainian)
        Ukr => tag(b"UKR "),

        // ქართული (Georgian)
        Kat => tag(b"KAT "),

        // العربية (Arabic)
        Arb => tag(b"ARA "),

        // हिन्दी (Hindi)
        Hin => tag(b"HIN "),

        // 日本語 (Japanese)
        Jpn => tag(b"JAN "),

        // עברית (Hebrew)
        Heb => tag(b"IWR "),

        // ייִדיש (Yiddish)
        Ydd => tag(b"JII "),

        // Polski (Polish)
        Pol => tag(b"PLK "),

        // አማርኛ (Amharic)
        Amh => tag(b"AMH "),

        // ትግርኛ (Tigrinya)
        Tir => tag(b"TGY "),

        // Basa Jawa (Javanese)
        Jav => tag(b"JAV "),

        // 한국어 (Korean)
        Kor => tag(b"KOR "),

        // Bokmål (Bokmal)
        Nob => None,

        // Nynorsk (Nynorsk)
        Nno => tag(b"NYN "),

        // Dansk (Danish)
        Dan => tag(b"DAN "),

        // Svenska (Swedish)
        Swe => tag(b"SVE "),

        // Suomi (Finnish)
        Fin => tag(b"FIN "),

        // Türkçe (Turkish)
        Tur => tag(b"TRK "),

        // Nederlands (Dutch)
        Nld => tag(b"NLD "),

        // Magyar (Hungarian)
        Hun => tag(b"HUN "),

        // Čeština (Czech)
        Ces => tag(b"CSY "),

        // Ελληνικά (Greek)
        Ell => tag(b"ELL "),

        // Български (Bulgarian)
        Bul => tag(b"BGR "),

        // Беларуская (Belarusian)
        Bel => tag(b"BEL "),

        // मराठी (Marathi)
        Mar => tag(b"MAR "),

        // ಕನ್ನಡ (Kannada)
        Kan => tag(b"KAN "),

        // Română (Romanian)
        Ron => tag(b"ROM "),

        // Slovenščina (Slovene)
        Slv => tag(b"SLV "),

        // Hrvatski (Croatian)
        Hrv => tag(b"HRV "),

        // Српски (Serbian)
        Srp => tag(b"SRB "),

        // Македонски (Macedonian)
        Mkd => tag(b"MKD "),

        // Lietuvių (Lithuanian)
        Lit => tag(b"LTH "),

        // Latviešu (Latvian)
        Lav => tag(b"LVI "),

        // Eesti (Estonian)
        Est => tag(b"ETI "),

        // தமிழ் (Tamil)
        Tam => tag(b"TAM "),

        // Tiếng Việt (Vietnamese)
        Vie => tag(b"VIT "),

        // اُردُو (Urdu)
        Urd => tag(b"URD "),

        // ภาษาไทย (Thai)
        Tha => tag(b"THA "),

        // ગુજરાતી (Gujarati)
        Guj => tag(b"GUJ "),

        // Oʻzbekcha (Uzbek)
        Uzb => tag(b"UZB "),

        // ਪੰਜਾਬੀ (Punjabi)
        Pan => tag(b"PAN "),

        // Azərbaycanca (Azerbaijani)
        Azj => tag(b"AZE "),

        // Bahasa Indonesia (Indonesian)
        Ind => tag(b"IND "),

        // తెలుగు (Telugu)
        Tel => tag(b"TEL "),

        // فارسی (Persian)
        Pes => tag(b"FAR "),

        // മലയാളം (Malayalam)
        Mal => tag(b"MAL "),

        // Hausa (Hausa)
        Hau => tag(b"HAU "),

        // ଓଡ଼ିଆ (Oriya)
        Ori => tag(b"ORI "),

        // မြန်မာစာ (Burmese)
        Mya => tag(b"BRM "),

        // भोजपुरी (Bhojpuri)
        Bho => tag(b"BHO "),

        // Tagalog (Tagalog)
        Tgl => tag(b"TGL "),

        // Yorùbá (Yoruba)
        Yor => tag(b"YBA "),

        // मैथिली (Maithili)
        Mai => tag(b"MTH "),

        // Oromoo (Oromo)
        Orm => tag(b"ORO "),

        // Igbo (Igbo)
        Ibo => tag(b"IBO "),

        // Cebuano (Cebuano)
        Ceb => tag(b"CEB "),

        // Kurdî (Kurdish)
        Kur => tag(b"KUR "),

        // Malagasy (Malagasy)
        Mlg => tag(b"MLG "),

        // سرائیکی (Saraiki)
        Skr => tag(b"SRK "),

        // नेपाली (Nepali)
        Nep => tag(b"NEP "),

        // සිංහල (Sinhalese)
        Sin => tag(b"SNH "),

        // ភាសាខ្មែរ (Khmer)
        Khm => tag(b"KHM "),

        // Türkmençe (Turkmen)
        Tuk => tag(b"TKM "),

        // Soomaaliga (Somali)
        Som => tag(b"SML "),

        // Chichewa (Chewa)
        Nya => tag(b"CHI "),

        // Akan (Akan)
        Aka => tag(b"AKA "),

        // IsiZulu (Zulu)
        Zul => tag(b"ZUL "),

        // Kinyarwanda (Kinyarwanda)
        Kin => tag(b"RUA "),

        // Kreyòl ayisyen (Haitian Creole)
        Hat => tag(b"HAI "),

        // Ilokano (Ilocano)
        Ilo => tag(b"ILO "),

        // Ikirundi (Rundi)
        Run => tag(b"RUN "),

        // ChiShona (Shona)
        Sna => tag(b"SNA0"),

        // ئۇيغۇرچە (Uyghur)
        Uig => tag(b"UYG "),

        // Afrikaans (Afrikaans)
        Afr => tag(b"AFK "),

        // Lingua Latina (Latin)
        Lat => tag(b"LAT "),

        // Slovenčina (Slovak)
        Slk => tag(b"SKY "),

        _ => None
    }
}