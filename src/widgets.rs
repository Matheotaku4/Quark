//! Composants dessinés à la main (icônes vectorielles, boutons, onglets).
//! Aucune dépendance à une police d'icônes : tout est tracé.

use crate::theme::{self, Palette};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    None,
    Play,
    Stop,
    Plus,
    Refresh,
    Trash,
    Gear,
    Info,
    /// Journal de session (page lignée).
    Log,
    List,
    Grid,
    Folder,
    /// Crayon (modifier la fiche, les tags, les notes).
    Pencil,
    /// Étoile pleine : jeu favori.
    Star,
    /// Étoile vide : jeu non favori.
    StarOff,
    /// Logo Steam, pour l'import de la bibliothèque.
    Steam,
    /// Croix de fermeture.
    Close,
    /// Maison de la page d'accueil.
    Home,
    /// Téléchargements en cours.
    Download,
    ArrowLeft,
    ArrowRight,
    /// Loupe pour la recherche et la liaison d'un jeu.
    Search,
    /// Entonnoir du menu de filtres, comme dans la bibliothèque Steam.
    Filter,
    /// Chaîne pour relier un jeu à une source d'images.
    Link,
    /// Flèche sortante d'un résultat web.
    ExternalLink,
}

impl Icon {
    /// Glyphe privé de la police Lucide embarquée. Les valeurs viennent de
    /// `lucide-static/font/lucide.css` (version 1.0) ; ne pas les remplacer par
    /// des caractères Unicode ordinaires, qui n'existent pas dans cette police.
    pub fn glyph(self) -> Option<&'static str> {
        Some(match self {
            Icon::None => return None,
            Icon::Play => "\u{e13c}",
            Icon::Stop => "\u{e167}",
            Icon::Plus => "\u{e13d}",
            Icon::Refresh => "\u{e145}",
            Icon::Trash => "\u{e18e}",
            Icon::Gear => "\u{e154}",
            Icon::Info => "\u{e0f9}",
            Icon::Log => "\u{e45f}",
            Icon::List => "\u{e106}",
            Icon::Grid => "\u{e0ff}",
            Icon::Folder => "\u{e0d7}",
            Icon::Pencil => "\u{e1f9}",
            Icon::Star => "\u{e176}",
            Icon::StarOff => "\u{e2b0}",
            // Lucide n'a pas de logo Steam : la manette représente la boutique.
            Icon::Steam => "\u{e0df}",
            Icon::Close => "\u{e1b2}",
            Icon::Home => "\u{e0f5}",
            Icon::Download => "\u{e0b2}",
            Icon::ArrowLeft => "\u{e048}",
            Icon::ArrowRight => "\u{e049}",
            Icon::Search => "\u{e151}",
            Icon::Filter => "\u{e0dc}",
            Icon::Link => "\u{e102}",
            Icon::ExternalLink => "\u{e0b9}",
        })
    }
}

/// Identifiant stable d'un contrôle animé : sa place et son libellé. Le bloc de
/// mise en page ne change pas, donc l'état de l'animation n'est jamais perdu.
fn control_id(ui: &egui::Ui, rect: Rect, label: &str, kind: &str) -> egui::Id {
    ui.id()
        .with((kind, label, rect.left().to_bits(), rect.top().to_bits()))
}

/// Contour rectangulaire (4 bandes) : évite `rect_stroke` et son `StrokeKind`.
pub fn border_rect(painter: &egui::Painter, rect: Rect, width: f32, color: Color32) {
    let w = width;
    painter.rect_filled(
        Rect::from_min_size(rect.left_top(), Vec2::new(rect.width(), w)),
        0.0,
        color,
    );
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(rect.left(), rect.bottom() - w),
            Vec2::new(rect.width(), w),
        ),
        0.0,
        color,
    );
    painter.rect_filled(
        Rect::from_min_size(rect.left_top(), Vec2::new(w, rect.height())),
        0.0,
        color,
    );
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(rect.right() - w, rect.top()),
            Vec2::new(w, rect.height()),
        ),
        0.0,
        color,
    );
}

/// Icône Lucide, peinte depuis la police embarquée. `ui` est nécessaire pour
/// que le texte soit posé dans le bon calque et découpé avec le reste du widget.
pub fn paint_icon(ui: &egui::Ui, rect: Rect, icon: Icon, color: Color32) {
    let Some(glyph) = icon.glyph() else {
        return;
    };
    let painter = ui.painter_at(rect);
    let size = rect.width().min(rect.height());
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        FontId::new(size, crate::fonts::icon_family()),
        color,
    );
}

/// Bouton principal plein (bleu Steam), avec Icône optionnelle.
pub fn accent_button(
    ui: &mut egui::Ui,
    size: Vec2,
    label: &str,
    pal: &Palette,
    enabled: bool,
    icon: Icon,
) -> Response {
    let label = crate::i18n::tr(label);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(size, sense);
    let pressed = enabled && resp.is_pointer_button_down_on();
    // Transition douce vers l'état survolé. egui ne redemande une image que
    // pendant la transition : au repos, aucun repaint supplémentaire.
    let hover = ui.ctx().animate_bool_with_time(
        control_id(ui, rect, &label, "accent"),
        enabled && resp.hovered(),
        0.12,
    );
    let base = if enabled { pal.accent } else { pal.surface };
    let bg = if pressed {
        pal.accent_pressed
    } else {
        theme::lerp_color(base, pal.accent_hover, hover)
    };
    // Texte noir sur un accent clair, blanc sur un accent sombre.
    let fg = if enabled {
        theme::on_accent(bg)
    } else {
        pal.text_faint
    };
    let painter = ui.painter_at(rect);
    // L'enfoncement est peint, jamais alloué : la mise en page ne bouge pas.
    painter.rect_filled(rect.shrink(if pressed { 1.0 } else { 0.0 }), 10.0, bg);
    if enabled && hover > 0.0 && !pressed {
        border_rect(
            &painter,
            rect.shrink(0.5),
            1.0,
            pal.accent_hover.gamma_multiply(0.30 * hover),
        );
    }

    let galley = painter.layout_no_wrap(
        label.clone(),
        FontId::new(14.0, crate::fonts::bold_family()),
        fg,
    );
    let icon_w = if icon == Icon::None { 0.0 } else { 15.0 };
    let gap = if icon_w > 0.0 { 9.0 } else { 0.0 };
    let total = icon_w + gap + galley.size().x;
    let x0 = rect.center().x - total / 2.0;
    if icon != Icon::None {
        paint_icon(
            ui,
            Rect::from_center_size(
                Pos2::new(x0 + icon_w / 2.0, rect.center().y),
                Vec2::splat(icon_w),
            ),
            icon,
            fg,
        );
    }
    painter.galley(
        Pos2::new(x0 + icon_w + gap, rect.center().y - galley.size().y / 2.0),
        galley,
        fg,
    );
    resp
}

/// Bouton d'action principal jaune (Continuer, Voir dans le navigateur). Le
/// noir sur jaune reste lisible dans tous les thèmes.
pub fn gold_button(
    ui: &mut egui::Ui,
    size: Vec2,
    label: &str,
    enabled: bool,
    icon: Icon,
) -> Response {
    let label = crate::i18n::tr(label);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(size, sense);
    let pressed = enabled && resp.is_pointer_button_down_on();
    let hover = ui.ctx().animate_bool_with_time(
        control_id(ui, rect, &label, "gold"),
        enabled && resp.hovered(),
        0.12,
    );
    let bg = if pressed {
        theme::GOLD.gamma_multiply(0.86)
    } else {
        theme::lerp_color(theme::GOLD, Color32::WHITE, hover * 0.18)
    };
    let fg = Color32::from_rgb(0x12, 0x12, 0x12);
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect.shrink(if pressed { 1.0 } else { 0.0 }), 10.0, bg);
    let galley = painter.layout_no_wrap(
        label.clone(),
        FontId::new(14.0, crate::fonts::bold_family()),
        fg,
    );
    let icon_w = if icon == Icon::None { 0.0 } else { 14.0 };
    let gap = if icon_w > 0.0 { 8.0 } else { 0.0 };
    let total = icon_w + gap + galley.size().x;
    let x0 = rect.center().x - total / 2.0;
    if icon != Icon::None {
        paint_icon(
            ui,
            Rect::from_center_size(
                Pos2::new(x0 + icon_w / 2.0, rect.center().y),
                Vec2::splat(icon_w),
            ),
            icon,
            fg,
        );
    }
    painter.galley(
        Pos2::new(x0 + icon_w + gap, rect.center().y - galley.size().y / 2.0),
        galley,
        fg,
    );
    resp
}

/// Bouton destructif (rouge).
pub fn danger_button(
    ui: &mut egui::Ui,
    size: Vec2,
    label: &str,
    pal: &Palette,
    enabled: bool,
    icon: Icon,
) -> Response {
    let label = crate::i18n::tr(label);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(size, sense);
    let fg = Color32::WHITE;
    let pressed = enabled && resp.is_pointer_button_down_on();
    let hover = ui.ctx().animate_bool_with_time(
        control_id(ui, rect, &label, "danger"),
        enabled && resp.hovered(),
        0.12,
    );
    let bg = if pressed {
        pal.danger.gamma_multiply(0.80)
    } else {
        theme::lerp_color(pal.danger, pal.danger.gamma_multiply(1.30), hover)
    };
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect.shrink(if pressed { 1.0 } else { 0.0 }), 4.0, bg);

    let galley = painter.layout_no_wrap(label.clone(), FontId::proportional(14.0), fg);
    let icon_w = if icon == Icon::None { 0.0 } else { 15.0 };
    let gap = if icon_w > 0.0 { 9.0 } else { 0.0 };
    let total = icon_w + gap + galley.size().x;
    let x0 = rect.center().x - total / 2.0;
    if icon != Icon::None {
        paint_icon(
            ui,
            Rect::from_center_size(
                Pos2::new(x0 + icon_w / 2.0, rect.center().y),
                Vec2::splat(icon_w),
            ),
            icon,
            fg,
        );
    }
    painter.galley(
        Pos2::new(x0 + icon_w + gap, rect.center().y - galley.size().y / 2.0),
        galley,
        fg,
    );
    resp
}

/// Bouton destructif discret : contour rouge, fond transparent, fond plein au
/// survol. Pour les actions secondaires comme « Supprimer » dans un pied de
/// fenêtre, sans le poids visuel du bouton rouge plein.
pub fn danger_ghost_button(
    ui: &mut egui::Ui,
    size: Vec2,
    label: &str,
    pal: &Palette,
    enabled: bool,
    icon: Icon,
) -> Response {
    let label = crate::i18n::tr(label);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(size, sense);
    let hover = ui.ctx().animate_bool_with_time(
        control_id(ui, rect, &label, "danger-ghost"),
        enabled && resp.hovered(),
        0.12,
    );
    let painter = ui.painter_at(rect);
    if hover > 0.0 {
        painter.rect_filled(rect, 8.0, pal.danger.gamma_multiply(0.20 * hover));
    }
    painter.rect_stroke(
        rect,
        8.0,
        Stroke::new(
            1.0_f32,
            pal.danger.gamma_multiply(if enabled { 1.0 } else { 0.45 }),
        ),
        egui::StrokeKind::Inside,
    );
    let ink = if enabled {
        theme::lerp_color(pal.danger, Color32::WHITE, hover * 0.18)
    } else {
        pal.text_faint
    };
    let galley = painter.layout_no_wrap(label.clone(), FontId::proportional(13.0), ink);
    let icon_w = if icon == Icon::None { 0.0 } else { 14.0 };
    let gap = if icon_w > 0.0 { 8.0 } else { 0.0 };
    let total = icon_w + gap + galley.size().x;
    let x0 = rect.center().x - total / 2.0;
    if icon != Icon::None {
        paint_icon(
            ui,
            Rect::from_center_size(
                Pos2::new(x0 + icon_w / 2.0, rect.center().y),
                Vec2::splat(icon_w),
            ),
            icon,
            ink,
        );
    }
    painter.galley(
        Pos2::new(x0 + icon_w + gap, rect.center().y - galley.size().y / 2.0),
        galley,
        ink,
    );
    resp
}

/// Bouton secondaire discret (contour au survol), largeur automatique.
pub fn ghost_button(ui: &mut egui::Ui, label: &str, pal: &Palette, icon: Icon) -> Response {
    let label = crate::i18n::tr(label);
    let font = FontId::proportional(12.5);
    let galley = ui
        .painter()
        .layout_no_wrap(label.clone(), font.clone(), pal.text);
    let icon_w = if icon == Icon::None { 0.0 } else { 13.0 };
    let gap = if icon_w > 0.0 { 8.0 } else { 0.0 };
    let size = Vec2::new(galley.size().x + icon_w + gap + 24.0, 32.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter_at(rect);
    let hover = ui.ctx().animate_bool_with_time(
        control_id(ui, rect, &label, "ghost"),
        resp.hovered(),
        0.12,
    );
    if hover > 0.0 {
        painter.rect_filled(rect, 8.0, pal.surface_hover.gamma_multiply(hover));
        border_rect(&painter, rect, 1.0, pal.border.gamma_multiply(hover));
    }
    let color = theme::lerp_color(pal.text_dim, pal.text, hover);
    let x0 = rect.center().x - (icon_w + gap + galley.size().x) / 2.0;
    if icon != Icon::None {
        paint_icon(
            ui,
            Rect::from_center_size(
                Pos2::new(x0 + icon_w / 2.0, rect.center().y),
                Vec2::splat(icon_w),
            ),
            icon,
            color,
        );
    }
    painter.galley(
        Pos2::new(x0 + icon_w + gap, rect.center().y - galley.size().y / 2.0),
        galley,
        color,
    );
    resp
}

/// Bouton icône carré (engrenage, +, corbeille…).
pub fn icon_button(
    ui: &mut egui::Ui,
    size: f32,
    icon: Icon,
    pal: &Palette,
    tooltip: &str,
) -> Response {
    let tooltip = crate::i18n::tr(tooltip);
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let painter = ui.painter_at(rect);
    let hover = ui.ctx().animate_bool_with_time(
        control_id(ui, rect, &tooltip, "icon"),
        resp.hovered(),
        0.12,
    );
    if hover > 0.0 {
        painter.rect_filled(rect, 8.0, pal.surface_hover.gamma_multiply(hover));
    }
    // Le symbole se resserre légèrement au survol : un effet de « prise », sans
    // aucune conséquence sur la mise en page.
    paint_icon(
        ui,
        rect.shrink(size * (0.24 + 0.03 * hover)),
        icon,
        theme::lerp_color(pal.text_dim, pal.text, hover),
    );
    if tooltip.is_empty() {
        resp
    } else {
        resp.on_hover_text(tooltip)
    }
}

/// Bouton icône carré coloré, pour les actions d'un jeu : poubelle rouge,
/// crayon vert, engrenage bleu. La couleur reste lisible sur le noir AMOLED et
/// sur une bannière claire grâce à une pastille sombre puis un liseré coloré.
pub fn tinted_icon_button(
    ui: &mut egui::Ui,
    size: f32,
    icon: Icon,
    color: Color32,
    tooltip: &str,
) -> Response {
    let tooltip = crate::i18n::tr(tooltip);
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let painter = ui.painter_at(rect);
    let hover = ui.ctx().animate_bool_with_time(
        control_id(ui, rect, &tooltip, "tinted"),
        resp.hovered(),
        0.12,
    );
    // Pastille sombre sous le symbole : posée sur une bannière claire, l'icône
    // colorée gardait peu de contraste. Le fond sombre la détache de l'image.
    painter.circle_filled(
        rect.center(),
        size * 0.5,
        Color32::from_black_alpha((105.0 + 45.0 * hover).round() as u8),
    );
    // Liseré de la couleur de l'action : le bouton se lit même de loin.
    painter.circle_stroke(
        rect.center(),
        size * 0.5 - 0.5,
        Stroke::new(1.0_f32, color.gamma_multiply(0.65 + 0.35 * hover)),
    );
    paint_icon(
        ui,
        rect.shrink(size * (0.29 + 0.02 * hover)),
        icon,
        theme::lerp_color(color, Color32::WHITE, hover * 0.30),
    );
    if tooltip.is_empty() {
        resp
    } else {
        resp.on_hover_text(tooltip)
    }
}

/// Entrée de menu contextuel : petite pastille sombre, icône colorée, libellé.
///
/// Un menu egui standard ne porte que du texte ; ces lignes donnent au clic
/// droit les mêmes icônes que la fiche. La pastille sombre garde le symbole
/// lisible même quand la fenêtre est posée sur une bannière claire.
pub fn menu_row(ui: &mut egui::Ui, pal: &Palette, icon: Icon, color: Color32, label: &str) -> bool {
    let label = crate::i18n::tr(label);
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width().max(206.0), 28.0),
        Sense::click(),
    );
    let painter = ui.painter_at(rect);
    let hover =
        ui.ctx()
            .animate_bool_with_time(ui.id().with(("quark_menu", &label)), resp.hovered(), 0.10);
    if hover > 0.0 {
        painter.rect_filled(rect, 5.0, pal.surface_hover.gamma_multiply(hover));
    }
    let center = Pos2::new(rect.left() + 18.0, rect.center().y);
    painter.circle_filled(center, 10.0, Color32::from_black_alpha(130));
    painter.circle_stroke(center, 9.5, Stroke::new(1.0_f32, color.gamma_multiply(0.6)));
    paint_icon(
        ui,
        Rect::from_center_size(center, Vec2::splat(12.0)),
        icon,
        color,
    );
    painter.text(
        Pos2::new(rect.left() + 36.0, rect.center().y),
        Align2::LEFT_CENTER,
        &label,
        FontId::proportional(12.5),
        theme::lerp_color(pal.text_dim, pal.text, 0.55 + 0.45 * hover),
    );
    resp.clicked()
}

/// Bouton à pastille avec libellé, icône et, au choix, un compteur (« Transferts · 2 »)
/// ou un raccourci clavier dans un petit cadre (« Ctrl K »). Remplace une icône
/// seule quand la place le permet : le libellé dit ce que fait le bouton.
pub fn pill_button(
    ui: &mut egui::Ui,
    pal: &Palette,
    icon: Icon,
    label: &str,
    count: usize,
    kbd: Option<&str>,
) -> Response {
    const ICON: f32 = 13.0;
    let label = crate::i18n::tr(label);
    let font = FontId::proportional(12.5);
    let label_galley = ui
        .painter()
        .layout_no_wrap(label.clone(), font.clone(), pal.text);
    let count_galley = (count > 0).then(|| {
        ui.painter()
            .layout_no_wrap(format!(" · {count}"), font.clone(), pal.accent)
    });
    let kbd_galley = kbd.map(|keys| {
        ui.painter()
            .layout_no_wrap(keys.to_string(), FontId::proportional(10.5), pal.text_dim)
    });
    let mut width = 12.0 + ICON + 8.0 + label_galley.size().x + 12.0;
    if let Some(galley) = &count_galley {
        width += galley.size().x;
    }
    if let Some(galley) = &kbd_galley {
        width += galley.size().x + 12.0 + 2.0;
    }
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 36.0), Sense::click());
    let painter = ui.painter_at(rect);
    let hover =
        ui.ctx()
            .animate_bool_with_time(control_id(ui, rect, &label, "pill"), resp.hovered(), 0.12);
    painter.rect_filled(
        rect,
        18.0,
        theme::lerp_color(pal.surface, pal.surface_hover, hover),
    );
    painter.rect_stroke(
        rect,
        18.0,
        Stroke::new(1.0_f32, pal.border),
        egui::StrokeKind::Inside,
    );
    let ink = theme::lerp_color(pal.text_dim, pal.text, hover);
    let mut x = rect.left() + 12.0;
    paint_icon(
        ui,
        Rect::from_center_size(
            Pos2::new(x + ICON / 2.0, rect.center().y),
            Vec2::splat(ICON),
        ),
        icon,
        ink,
    );
    x += ICON + 8.0;
    let label_w = label_galley.size().x;
    painter.galley(
        Pos2::new(x, rect.center().y - label_galley.size().y / 2.0),
        label_galley,
        ink,
    );
    x += label_w;
    if let Some(galley) = count_galley {
        let w = galley.size().x;
        painter.galley(
            Pos2::new(x, rect.center().y - galley.size().y / 2.0),
            galley,
            pal.accent,
        );
        x += w;
    }
    if let Some(galley) = kbd_galley {
        let box_rect = Rect::from_min_size(
            Pos2::new(x + 10.0, rect.center().y - 9.0),
            Vec2::new(galley.size().x + 12.0, 18.0),
        );
        painter.rect_stroke(
            box_rect,
            5.0,
            Stroke::new(1.0_f32, pal.border),
            egui::StrokeKind::Inside,
        );
        painter.galley(
            Pos2::new(
                box_rect.center().x - galley.size().x / 2.0,
                box_rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            pal.text_dim,
        );
    }
    resp
}

/// Contrôle segmenté (par exemple Grille | Liste) : le libellé reste visible,
/// contrairement à des icônes seules dont le sens ne passait que par
/// l'infobulle. Renvoie l'index du segment cliqué, jamais celui déjà choisi.
pub fn segmented(
    ui: &mut egui::Ui,
    pal: &Palette,
    items: &[(Icon, &str)],
    selected: usize,
) -> Option<usize> {
    const HEIGHT: f32 = 32.0;
    let mut font = FontId::proportional(12.0);
    let mut galleys: Vec<_> = items
        .iter()
        .map(|(_, label)| {
            ui.painter()
                .layout_no_wrap(crate::i18n::tr(label), font.clone(), pal.text)
        })
        .collect();
    let mut widths: Vec<f32> = galleys
        .iter()
        .map(|galley| galley.size().x + 13.0 + 6.0 + 20.0)
        .collect();
    let mut total = widths.iter().sum::<f32>() + 4.0;
    // Cinq statuts tiennent difficilement dans une carte de 320 px. Quand les
    // segments n'ont pas d'icône et dépassent la largeur disponible, on retire
    // l'icône (déjà absente) et on resserre la police et le padding.
    let compact = total > ui.available_width() && items.iter().all(|(icon, _)| *icon == Icon::None);
    if compact {
        font = FontId::proportional(11.0);
        galleys = items
            .iter()
            .map(|(_, label)| {
                ui.painter()
                    .layout_no_wrap(crate::i18n::tr(label), font.clone(), pal.text)
            })
            .collect();
        widths = galleys
            .iter()
            .map(|galley| galley.size().x + 14.0)
            .collect();
        total = widths.iter().sum::<f32>() + 4.0;
    }
    let icon_w = if compact { 0.0 } else { 13.0 };
    let gap = if compact { 0.0 } else { 6.0 };
    let (outer, _) = ui.allocate_exact_size(Vec2::new(total, HEIGHT), Sense::hover());
    let painter = ui.painter_at(outer);
    painter.rect_filled(outer, 10.0, pal.surface);
    painter.rect_stroke(
        outer,
        10.0,
        Stroke::new(1.0_f32, pal.border),
        egui::StrokeKind::Inside,
    );
    let mut clicked = None;
    let group = items
        .iter()
        .map(|(_, label)| *label)
        .collect::<Vec<_>>()
        .join("|");
    let mut x = outer.left() + 2.0;
    for (index, ((icon, _), galley)) in items.iter().zip(galleys).enumerate() {
        let rect = Rect::from_min_size(
            Pos2::new(x, outer.top() + 2.0),
            Vec2::new(widths[index], HEIGHT - 4.0),
        );
        x += widths[index];
        let resp = ui.interact(
            rect,
            ui.id().with(("quark_segment", group.as_str(), index)),
            Sense::click(),
        );
        let on = index == selected;
        let hover = ui.ctx().animate_bool_with_time(
            ui.id().with(("quark_segment_hover", group.as_str(), index)),
            resp.hovered() && !on,
            0.12,
        );
        if on {
            painter.rect_filled(rect, 8.0, pal.accent);
        } else if hover > 0.0 {
            painter.rect_filled(rect, 8.0, pal.surface_hover.gamma_multiply(hover));
        }
        let ink = if on {
            theme::on_accent(pal.accent)
        } else {
            theme::lerp_color(pal.text_dim, pal.text, hover)
        };
        let content = icon_w + gap + galley.size().x;
        let x0 = rect.center().x - content / 2.0;
        if icon_w > 0.0 && *icon != Icon::None {
            paint_icon(
                ui,
                Rect::from_center_size(
                    Pos2::new(x0 + icon_w / 2.0, rect.center().y),
                    Vec2::splat(icon_w),
                ),
                *icon,
                ink,
            );
        }
        painter.galley(
            Pos2::new(x0 + icon_w + gap, rect.center().y - galley.size().y / 2.0),
            galley,
            ink,
        );
        if resp.clicked() && !on {
            clicked = Some(index);
        }
    }
    clicked
}

/// Bouton « ··· » : trois points tracés à la main (la police d'icônes embarquée
/// n'a pas de glyphe fiable pour eux). Sert à ouvrir un menu d'actions.
pub fn more_button(ui: &mut egui::Ui, size: f32, pal: &Palette, tooltip: &str) -> Response {
    let tooltip = crate::i18n::tr(tooltip);
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let painter = ui.painter_at(rect);
    let hover = ui.ctx().animate_bool_with_time(
        control_id(ui, rect, &tooltip, "more"),
        resp.hovered(),
        0.12,
    );
    painter.rect_filled(
        rect,
        10.0,
        theme::lerp_color(pal.surface, pal.surface_hover, hover),
    );
    painter.rect_stroke(
        rect,
        10.0,
        Stroke::new(1.0_f32, pal.border),
        egui::StrokeKind::Inside,
    );
    let ink = theme::lerp_color(pal.text_dim, pal.text, hover);
    for offset in [-5.5_f32, 0.0, 5.5] {
        painter.circle_filled(rect.center() + Vec2::new(offset, 0.0), 1.7, ink);
    }
    if tooltip.is_empty() {
        resp
    } else {
        resp.on_hover_text(tooltip)
    }
}

/// Onglet de la barre supérieure (Boutique / Bibliothèque).
pub fn tab(ui: &mut egui::Ui, label: &str, selected: bool, pal: &Palette) -> Response {
    let label = crate::i18n::tr(label);
    let text = label.to_uppercase();
    let font = FontId::new(12.5, crate::fonts::bold_family());
    let width = ui
        .painter()
        .layout_no_wrap(text.clone(), font.clone(), pal.text)
        .size()
        .x;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width + 18.0, 40.0), Sense::click());
    let painter = ui.painter_at(rect);
    let hover = ui.ctx().animate_bool_with_time(
        ui.id().with(("quark-tab-hover", &label)),
        resp.hovered(),
        0.12,
    );
    // Le trait d'onglet s'étire depuis le centre quand l'onglet est choisi.
    let mark =
        ui.ctx()
            .animate_bool_with_time(ui.id().with(("quark-tab-mark", &label)), selected, 0.16);
    let color = if selected {
        pal.text
    } else {
        theme::lerp_color(pal.text_dim, pal.text, hover)
    };
    painter.text(rect.center(), Align2::CENTER_CENTER, text, font, color);
    if mark > 0.0 {
        let width = (rect.width() - 4.0) * (0.55 + 0.45 * mark);
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(
                    rect.center().x - width / 2.0,
                    rect.bottom() - 2.5 - (1.0 - mark) * 2.0,
                ),
                Vec2::new(width, 2.5),
            ),
            1.25,
            pal.accent.gamma_multiply(mark),
        );
    }
    resp
}

/// Vignette de secours quand le jeu n'a pas d'artwork.
/// Vignette de secours d'un jeu sans artwork : la première lettre du titre sur
/// la couleur assignée au jeu. Le rectangle reçu est exactement celui de
/// l'artwork, donc la vignette occupe toujours la même place.
pub fn monogram(painter: &egui::Painter, rect: Rect, text: &str, fill: Color32, _pal: &Palette) {
    // Coins arrondis comme les vignettes d'images : la pastille de secours ne
    // se distingue plus du reste de la liste.
    painter.rect_filled(rect, 10.0, fill);
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        FontId::proportional((rect.height() * 0.36).clamp(13.0, 120.0)),
        // Sur une couleur claire, une lettre blanche serait illisible.
        theme::on_surface(fill),
    );
}

/// Pastille de filtre (vue, tri, tag, statut) : pleine quand elle est active.
pub fn chip(ui: &mut egui::Ui, label: &str, selected: bool, pal: &Palette) -> bool {
    chip_colored(ui, label, selected, pal, pal.accent)
}

/// Même pastille avec une couleur propre (statuts, tags).
pub fn chip_colored(
    ui: &mut egui::Ui,
    label: &str,
    selected: bool,
    pal: &Palette,
    accent: Color32,
) -> bool {
    let ink = if selected {
        theme::on_accent(accent)
    } else {
        pal.text_dim
    };
    ui.add(
        egui::Button::new(crate::i18n::rich(label).size(12.0).color(ink))
            .fill(if selected { accent } else { pal.surface })
            .stroke(egui::Stroke::new(
                1.0_f32,
                if selected { accent } else { pal.border },
            ))
            .corner_radius(16.0)
            .min_size(Vec2::new(0.0, 30.0)),
    )
    .clicked()
}

/// Ligne de réglage complète : intitulé et explication à gauche, interrupteur à
/// droite, fond qui s'éclaire au survol. Toute la ligne répond au clic.
pub fn setting_toggle(
    ui: &mut egui::Ui,
    pal: &Palette,
    title: &str,
    hint: Option<&str>,
    value: &mut bool,
    id_salt: impl std::hash::Hash,
) -> Response {
    let id = ui.id().with(("quark-setting", &id_salt));
    let width = ui.available_width();
    let height = if hint.is_some() { 46.0 } else { 34.0 };
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("row-hover"), response.hovered(), 0.12);
    let on = ui
        .ctx()
        .animate_bool_with_time(id.with("toggle"), *value, 0.18);
    if hover > 0.0 {
        ui.painter()
            .rect_filled(rect, 7.0, pal.surface_hover.gamma_multiply(hover));
    }
    // Texte à gauche, largeur limitée pour laisser la place à l'interrupteur.
    let text_rect = Rect::from_min_max(
        Pos2::new(rect.left() + 10.0, rect.top() + 4.0),
        Pos2::new(rect.right() - 66.0, rect.bottom() - 4.0),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(text_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.spacing_mut().item_spacing.y = 1.0;
    child.label(crate::i18n::rich(title).size(12.5).color(pal.text));
    if let Some(hint) = hint {
        child
            .add(egui::Label::new(crate::i18n::rich(hint).size(10.5).color(pal.text_faint)).wrap());
    }
    // Interrupteur peint, sans widget interactif imbriqué : toute la ligne
    // réagit au clic, sans double bascule.
    let track = Rect::from_center_size(
        Pos2::new(rect.right() - 31.0, rect.center().y),
        Vec2::new(42.0, 23.0),
    );
    let painter = ui.painter_at(rect);
    painter.rect_filled(
        track,
        track.height() / 2.0,
        theme::lerp_color(pal.border, pal.accent, on),
    );
    let knob_x = track.left() + track.height() / 2.0 + (track.width() - track.height()) * on;
    painter.circle_filled(
        Pos2::new(knob_x, track.center().y),
        track.height() / 2.0 - 3.0,
        if on > 0.5 {
            theme::on_accent(pal.accent)
        } else {
            pal.text_dim
        },
    );
    if response.clicked()
        || (response.has_focus()
            && ui.input(|input| {
                input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
            }))
    {
        *value = !*value;
        response.mark_changed();
    }
    response
}

/// Entrée « Accueil » : première ligne de la liste, pleine largeur comme dans
/// Steam. Elle suit la même grille que les rangées de jeux (carré de 36 px à
/// 16 px du bord, titre à 66 px, repère d'accent à gauche), si bien que la liste
/// s'aligne du haut en bas. Le « + » d'ajout est posé à droite de la même ligne.
/// Renvoie `(accueil cliqué, ajout cliqué)`.
pub fn home_row(ui: &mut egui::Ui, selected: bool, pal: &Palette) -> (bool, bool) {
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 46.0), Sense::click());
    let painter = ui.painter_at(rect);
    let id = ui.id().with("quark-home-row");
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), resp.hovered(), 0.12);
    let mark = ui
        .ctx()
        .animate_bool_with_time(id.with("selected"), selected, 0.16);
    if hover > 0.0 {
        painter.rect_filled(
            rect,
            10.0,
            theme::lerp_color(Color32::TRANSPARENT, pal.surface_hover, hover),
        );
    }
    if mark > 0.0 {
        painter.rect_filled(rect, 10.0, pal.surface_hover.gamma_multiply(mark));
        let height = (rect.height() - 10.0) * mark;
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(rect.left(), rect.center().y - height / 2.0),
                Vec2::new(3.0, height),
            ),
            1.5,
            pal.accent.gamma_multiply(mark),
        );
    }
    let tile = Rect::from_min_size(
        Pos2::new(rect.left() + 16.0, rect.center().y - 18.0),
        Vec2::splat(36.0),
    );
    painter.rect_filled(
        tile,
        7.0,
        theme::lerp_color(pal.surface, pal.surface_hover, hover),
    );
    let ink = if selected {
        pal.text
    } else {
        theme::lerp_color(pal.text_dim, pal.text, hover)
    };
    paint_icon(
        ui,
        Rect::from_center_size(tile.center(), Vec2::splat(17.0)),
        Icon::Home,
        if selected { pal.accent } else { ink },
    );
    painter.text(
        Pos2::new(rect.left() + 66.0, rect.center().y),
        Align2::LEFT_CENTER,
        crate::i18n::tr("Accueil"),
        FontId::proportional(13.5),
        ink,
    );

    // Le « + » est enregistré après la ligne : il reste au-dessus d'elle pour
    // les clics.
    let plus_rect = Rect::from_center_size(
        Pos2::new(rect.right() - 26.0, rect.center().y),
        Vec2::splat(28.0),
    );
    let plus = ui.interact(plus_rect, id.with("plus"), Sense::click());
    let plus_hover = ui
        .ctx()
        .animate_bool_with_time(id.with("plus-hover"), plus.hovered(), 0.12);
    painter.rect_filled(
        plus_rect,
        6.0,
        theme::lerp_color(Color32::TRANSPARENT, pal.surface_hover, plus_hover),
    );
    paint_icon(
        ui,
        Rect::from_center_size(plus_rect.center(), Vec2::splat(15.0)),
        Icon::Plus,
        theme::lerp_color(pal.text_dim, pal.text, plus_hover),
    );
    let plus = plus.on_hover_text(crate::i18n::tr("Ajouter un jeu"));
    (resp.clicked() && !plus.clicked(), plus.clicked())
}

/// Bouton icône posé **sur une image** (bannière) : un fond translucide rend le
/// symbole lisible quelle que soit l'image, et sa couleur s'oppose au fond
/// (sombre sur une bannière claire, clair sur une bannière sombre).
/// Petit ⓘ ) côté d'un réglage : l'explication remplace les lignes de conseil
/// qui encombraient les fenêtres. Le texte complet s'affiche au survol.
pub fn info_button(ui: &mut egui::Ui, pal: &Palette, text: &str) -> Response {
    icon_button(ui, 16.0, Icon::Info, pal, text)
}

/// Applique le style des champs de saisie : fond plus sombre que la carte,
/// contour fin et accent au focus. Le style reste limité à l'`Ui` courante.
pub fn style_text_inputs(ui: &mut egui::Ui, pal: &Palette) {
    let visuals = &mut ui.style_mut().visuals;
    visuals.extreme_bg_color = pal.bg;
    visuals.selection.bg_fill = pal.accent.gamma_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0_f32, pal.text);
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.bg_fill = pal.bg;
        widget.weak_bg_fill = pal.bg;
        widget.bg_stroke = Stroke::new(1.0_f32, pal.border);
        widget.corner_radius = egui::CornerRadius::same(8);
    }
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, pal.accent);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, pal.border);
}

/// Carte de réglages : fond `surface`, contour fin, coins de 12 px et titre de
/// 15 px. Le ⓘ du titre remplace les anciennes barres de section.
pub fn card<R>(
    ui: &mut egui::Ui,
    pal: &Palette,
    title: &str,
    hint: Option<&str>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::new()
        .fill(pal.surface)
        .stroke(Stroke::new(1.0_f32, pal.border))
        .corner_radius(12.0)
        .inner_margin(egui::Margin::symmetric(18, 16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 12.0;
            ui.horizontal(|ui| {
                ui.label(crate::i18n::rich(title).size(15.0).strong().color(pal.text));
                if let Some(hint) = hint {
                    info_button(ui, pal, hint);
                }
            });
            add_contents(ui)
        })
        .inner
}

/// Libellé de champ de 12 px suivi de son contrôle, avec 6 px d'écart.
pub fn field<R>(
    ui: &mut egui::Ui,
    pal: &Palette,
    label: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        ui.label(crate::i18n::rich(label).size(12.0).color(pal.text_dim));
        style_text_inputs(ui, pal);
        add_contents(ui)
    })
    .inner
}

/// Champ de chemin pleine largeur suivi du bouton « Parcourir… ». Renvoie
/// `(modifié, parcourir cliqué)`.
pub fn path_field(
    ui: &mut egui::Ui,
    pal: &Palette,
    value: &mut String,
    hint: &str,
    browse_label: &str,
) -> (bool, bool) {
    let mut changed = false;
    let mut browse = false;
    ui.horizontal(|ui| {
        style_text_inputs(ui, pal);
        let field_width = (ui.available_width() - 122.0).max(140.0);
        let response = ui.add_sized(
            Vec2::new(field_width, 34.0),
            egui::TextEdit::singleline(value).hint_text(crate::i18n::tr(hint)),
        );
        changed = response.changed();
        ui.add_space(8.0);
        browse = ghost_button(ui, browse_label, pal, Icon::Folder).clicked();
    });
    (changed, browse)
}

/// Ligne d'option : titre et description à gauche, interrupteur à droite.
/// Toute la ligne bascule la valeur, comme `setting_toggle`, avec le rendu des
/// paramètres refondus (fond `bg`, rayon 10, contour fin).
pub fn option_row(
    ui: &mut egui::Ui,
    pal: &Palette,
    title: &str,
    description: Option<&str>,
    value: &mut bool,
    id_salt: impl std::hash::Hash,
) -> Response {
    let id = ui.id().with(("quark_option_row", &id_salt));
    let height = if description.is_some() { 64.0 } else { 48.0 };
    let (rect, mut response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), response.hovered(), 0.12);
    let on = ui
        .ctx()
        .animate_bool_with_time(id.with("toggle"), *value, 0.18);
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 10.0, pal.bg);
    painter.rect_stroke(
        rect,
        10.0,
        Stroke::new(1.0_f32, pal.border),
        egui::StrokeKind::Inside,
    );
    if hover > 0.0 {
        painter.rect_filled(rect, 10.0, pal.surface_hover.gamma_multiply(0.35 * hover));
    }
    let text_rect = Rect::from_min_max(
        Pos2::new(rect.left() + 14.0, rect.top() + 8.0),
        Pos2::new(rect.right() - 72.0, rect.bottom() - 8.0),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(text_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.spacing_mut().item_spacing.y = 2.0;
    child.label(crate::i18n::rich(title).size(13.0).color(pal.text));
    if let Some(description) = description {
        let label = child.add(
            egui::Label::new(
                crate::i18n::rich(description)
                    .size(12.0)
                    .color(pal.text_dim),
            )
            .truncate(),
        );
        label.on_hover_text(description);
    }
    let track = Rect::from_center_size(
        Pos2::new(rect.right() - 31.0, rect.center().y),
        Vec2::new(42.0, 23.0),
    );
    painter.rect_filled(
        track,
        track.height() / 2.0,
        theme::lerp_color(pal.border, pal.accent, on),
    );
    let knob_x = track.left() + track.height() / 2.0 + (track.width() - track.height()) * on;
    painter.circle_filled(
        Pos2::new(knob_x, track.center().y),
        track.height() / 2.0 - 3.0,
        if on > 0.5 {
            theme::on_accent(pal.accent)
        } else {
            pal.text_dim
        },
    );
    if response.clicked()
        || (response.has_focus()
            && ui.input(|input| {
                input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
            }))
    {
        *value = !*value;
        response.mark_changed();
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Variante « statut » de `option_row` : une ligne verte « Trouvé · … » (ou
/// grise « Introuvable ») remplace la description. Le chemin complet reste
/// dans l'infobulle.
pub struct OptionRowStatus<'a> {
    pub found: bool,
    pub detail: Option<&'a str>,
}

pub fn option_row_status(
    ui: &mut egui::Ui,
    pal: &Palette,
    title: &str,
    status: &str,
    state: OptionRowStatus<'_>,
    value: &mut bool,
    id_salt: impl std::hash::Hash,
) -> Response {
    let OptionRowStatus { found, detail } = state;
    let id = ui.id().with(("quark_option_status", &id_salt));
    let (rect, mut response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 64.0), Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), response.hovered(), 0.12);
    let on = ui
        .ctx()
        .animate_bool_with_time(id.with("toggle"), *value, 0.18);
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 10.0, pal.bg);
    painter.rect_stroke(
        rect,
        10.0,
        Stroke::new(1.0_f32, pal.border),
        egui::StrokeKind::Inside,
    );
    if hover > 0.0 {
        painter.rect_filled(rect, 10.0, pal.surface_hover.gamma_multiply(0.35 * hover));
    }
    let text_rect = Rect::from_min_max(
        Pos2::new(rect.left() + 14.0, rect.top() + 8.0),
        Pos2::new(rect.right() - 72.0, rect.bottom() - 8.0),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(text_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.spacing_mut().item_spacing.y = 2.0;
    child.label(crate::i18n::rich(title).size(13.0).color(pal.text));
    let color = if found {
        Color32::from_rgb(0x5A, 0xD6, 0x8A)
    } else {
        pal.text_faint
    };
    let status_label =
        child.add(egui::Label::new(crate::i18n::rich(status).size(12.0).color(color)).truncate());
    if let Some(detail) = detail {
        status_label.on_hover_text(detail);
    }
    let track = Rect::from_center_size(
        Pos2::new(rect.right() - 31.0, rect.center().y),
        Vec2::new(42.0, 23.0),
    );
    painter.rect_filled(
        track,
        track.height() / 2.0,
        theme::lerp_color(pal.border, pal.accent, on),
    );
    let knob_x = track.left() + track.height() / 2.0 + (track.width() - track.height()) * on;
    painter.circle_filled(
        Pos2::new(knob_x, track.center().y),
        track.height() / 2.0 - 3.0,
        if on > 0.5 {
            theme::on_accent(pal.accent)
        } else {
            pal.text_dim
        },
    );
    if response.clicked()
        || (response.has_focus()
            && ui.input(|input| {
                input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space)
            }))
    {
        *value = !*value;
        response.mark_changed();
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Deux colonnes égales, gouttière de 16 px, empilées sous 760 px de largeur.
/// Barre de pied de fenêtre : bordure supérieure et padding 28×14. Le contenu
/// est placé après la zone défilante, donc jamais recouvert par elle.
/// Variante de `footer_bar` avec une largeur imposée : dans un modal, le pied
/// ne doit pas élargir la fenêtre en réclamant toute la largeur disponible.
pub fn footer_bar_sized<R>(
    ui: &mut egui::Ui,
    pal: &Palette,
    width: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let response = egui::Frame::new()
        .fill(pal.bg)
        .inner_margin(egui::Margin::symmetric(28, 14))
        .show(ui, |ui| {
            // `width` est la largeur extérieure du pied : on retire les deux
            // marges de 28 px pour que le fond ne déborde pas de la carte.
            ui.set_width((width - 56.0).max(0.0));
            add_contents(ui)
        });
    let rect = response.response.rect;
    ui.painter().rect_filled(
        Rect::from_min_size(rect.min, Vec2::new(rect.width(), 1.0)),
        0.0,
        pal.border,
    );
    response.inner
}

/// Entrée de navigation latérale : hauteur 36, rayon 8, fond `row_selected` et
/// texte d'accent pour la page active, sans contour.
pub fn nav_entry(ui: &mut egui::Ui, pal: &Palette, label: &str, selected: bool) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 36.0), Sense::click());
    let hover = ui.ctx().animate_bool_with_time(
        ui.id().with(("quark_nav", label)),
        response.hovered() && !selected,
        0.12,
    );
    let painter = ui.painter_at(rect);
    if selected {
        painter.rect_filled(rect, 8.0, pal.row_selected);
    } else if hover > 0.0 {
        painter.rect_filled(rect, 8.0, pal.surface_hover.gamma_multiply(hover));
    }
    let ink = if selected {
        pal.accent
    } else {
        theme::lerp_color(pal.text_dim, pal.text, hover)
    };
    let font = if selected {
        FontId::new(13.0, crate::fonts::bold_family())
    } else {
        FontId::proportional(13.0)
    };
    painter.text(
        Pos2::new(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        crate::i18n::tr(label),
        font,
        ink,
    );
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}

/// Onglet de contenu (fiche de jeu) : casse naturelle et soulignement bleu,
/// pour distinguer la navigation interne de la barre du haut.
pub fn detail_tab(ui: &mut egui::Ui, label: &str, selected: bool, pal: &Palette) -> egui::Response {
    let label = crate::i18n::tr(label);
    let font = egui::FontId::proportional(14.0);
    let galley = ui
        .painter()
        .layout_no_wrap(label.clone(), font.clone(), pal.text);
    let width = galley.size().x + 30.0;
    let (rect, response) =
        ui.allocate_exact_size(egui::Vec2::new(width, 40.0), egui::Sense::click());
    let color = if selected || response.hovered() {
        pal.text
    } else {
        pal.text_dim
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        &label,
        font,
        color,
    );
    if selected {
        let bar = egui::Rect::from_min_size(
            egui::Pos2::new(rect.left() + 10.0, rect.bottom() - 2.0),
            egui::Vec2::new(rect.width() - 20.0, 2.0),
        );
        ui.painter().rect_filled(bar, 1.0, pal.accent);
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
