use orbtk::prelude::*;
use std::cell::Cell;
use std::collections::BTreeMap;

/// Stacks visual the children widgets vertical or horizontal.
#[derive(Default, IntoLayout)]
pub struct DistributeLayout {
    desired_size: RefCell<DirtySize>,
    old_alignment: Cell<(Alignment, Alignment)>,
}

impl DistributeLayout {
    pub fn new() -> Self {
        DistributeLayout::default()
    }

    pub fn set_dirty(&self, dirty: bool) {
        self.desired_size.borrow_mut().set_dirty(dirty);
    }
}

impl Layout for DistributeLayout {
    fn measure(
        &self,
        render_context_2_d: &mut RenderContext2D,
        entity: Entity,
        ecm: &mut EntityComponentManager<Tree, StringComponentStore>,
        layouts: &BTreeMap<Entity, Box<dyn Layout>>,
        theme: &Theme,
    ) -> DirtySize {
        if component::<Visibility>(ecm, entity, "visibility") == Visibility::Collapsed {
            let mut desired = self.desired_size.borrow_mut();
            desired.set_size(0.0, 0.0);
            return *desired;
        }

        let halign: Alignment = component(ecm, entity, "h_align");
        let valign: Alignment = component(ecm, entity, "v_align");
        let (old_valign, old_halign) = self.old_alignment.get();

        if halign != old_halign || valign != old_valign {
            self.set_dirty(true);
        }

        let mut dirty = false;
        let mut desired_size: (f64, f64) = (0.0, 0.0);

        let nchildren = ecm.entity_store().children[&entity].len();
        for index in 0..nchildren {
            let child = ecm.entity_store().children[&entity][index];
            if let Some(child_layout) = layouts.get(&child) {
                let dirty = child_layout
                    .measure(render_context_2_d, child, ecm, layouts, theme)
                    .dirty()
                    || self.desired_size.borrow().dirty();

                self.desired_size.borrow_mut().set_dirty(dirty);
            }
        }

        self.set_dirty(dirty);

        let mut desired = self.desired_size.borrow_mut();
        desired.set_size(desired_size.0, desired_size.1);
        *desired
    }

    fn arrange(
        &self,
        render_context_2_d: &mut RenderContext2D,
        parent_size: (f64, f64),
        entity: Entity,
        ecm: &mut EntityComponentManager<Tree, StringComponentStore>,
        layouts: &BTreeMap<Entity, Box<dyn Layout>>,
        theme: &Theme,
    ) -> (f64, f64) {
        if component::<Visibility>(ecm, entity, "visibility") == Visibility::Collapsed {
            self.desired_size.borrow_mut().set_size(0.0, 0.0);
            return (0.0, 0.0);
        }

        let slots_size = (64.0, 64.0);

        let cols = (parent_size.0 / slots_size.0) as u32;
        let mut col = 0;
        let mut row = 0;

        for index in 0..ecm.entity_store().children[&entity].len() {
            let child = ecm.entity_store().children[&entity][index];
            if let Some(child_layout) = layouts.get(&child) {
                child_layout.arrange(
                    render_context_2_d,
                    (slots_size.0, slots_size.1),
                    child,
                    ecm,
                    layouts,
                    theme,
                );
            }
            if let Ok(child_bounds) = ecm
                .component_store_mut()
                .get_mut::<Rectangle>("bounds", child)
            {
                let x_diff = slots_size.0 - child_bounds.width();
                let y_diff = slots_size.1 - child_bounds.height();
                child_bounds.set_x(col as f64 * slots_size.0 + x_diff / 2.0);
                child_bounds.set_y(row as f64 * slots_size.1 + y_diff / 2.0);
                col += 1;
                if col == cols {
                    col = 0;
                    row += 1;
                }
            }
            mark_as_dirty("bounds", child, ecm);
        }

        self.desired_size.borrow_mut().set_dirty(false);
        let size = (cols as f64 * slots_size.0, row as f64 * slots_size.1);
        if let Some(bounds) = component_try_mut::<Rectangle>(ecm, entity, "bounds") {
            bounds.set_width(size.0);
            bounds.set_height(size.1);
        }

        mark_as_dirty("bounds", entity, ecm);
        size
    }
}

fn component<C: Component + Clone>(
    ecm: &mut EntityComponentManager<Tree, StringComponentStore>,
    entity: Entity,
    component: &str,
) -> C {
    ecm.component_store()
        .get::<C>(component, entity)
        .unwrap()
        .clone()
}

fn component_try_mut<'a, C: Component>(
    ecm: &'a mut EntityComponentManager<Tree, StringComponentStore>,
    entity: Entity,
    component: &str,
) -> Option<&'a mut C> {
    ecm.component_store_mut()
        .get_mut::<C>(component, entity)
        .ok()
}
