use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;
use crate::entities::entity::EntityLogic;
use macroquad::math::Vec2;

///Scene node to implement a scene graph.
///It allow access to childs, parent and holds
///a pointer to anything considered as an entity 
///from trait EntityLogic
#[derive(Debug)]
pub struct SceneNode {
    value: Box<dyn EntityLogic>,
    childs: RefCell<Vec<Rc<RefCell<SceneNode>>>>,   //RefCell allow borrow checker of data at runtime
                                                    //This ensure some data can be borrowed mutable or
                                                    //inmutable at runtime
                                                    //
                                                    //rc allow to share multiple ownership of same data
    parent: RefCell<Weak<RefCell<SceneNode>>>,
}

impl SceneNode {
    pub fn new(value: Box<dyn EntityLogic>) -> Self {
        Self {
            value,
            childs: RefCell::new(vec![]),
            parent: RefCell::new(Weak::new()),
        }
    }
    pub fn add_child(this: &Rc<RefCell<Self>>, child: &Rc<RefCell<Self>>) {
        *child.borrow_mut().parent.borrow_mut() = Rc::downgrade(this);
        this.borrow().childs.borrow_mut().push(child.clone());
    }


    pub fn update(this: &Rc<RefCell<Self>>, dt: f32) {
        // Extraemos el Rc<dyn EntityLogic> en una variable temporal,
        // para evitar mantener el borrow vivo mientras llamamos update_current
        let _value_update_fn = {
            let node = this.borrow();
            // copiamos un puntero (no clonamos, solo extraemos una referencia a la función)
            let ptr = &*node.value as *const dyn EntityLogic;
            unsafe { &*ptr }
        };

        // Llamamos a la función desde fuera del borrow original
        // Ahora no hay `RefMut` activo sobre `this`
        // Nota: aquí necesitas pasar `&Rc<RefCell<SceneNode>>`, no un borrow interno
        let mut_ref = unsafe {
            // unsafe porque estamos simulando un `&mut` a `value` sin préstamo activo
            // usamos solo porque sabemos que `update_current` no accede a nada más de `this`
            &mut *(this.borrow_mut().value.as_mut() as *mut dyn EntityLogic)
        };

        mut_ref.update_current(this, dt);

        // Ahora es seguro iterar los hijos
        let children = this.borrow().childs.borrow().clone();
        for child in children {
            Self::update(&child, dt);
        }
    }

    pub fn draw(this: &Rc<RefCell<Self>>) {
        this.borrow().value.draw_current();
        for child in this.borrow().childs.borrow().iter() {
            Self::draw(child);
        }
    } 

    pub fn get_parent_position(this: &Rc<RefCell<Self>>) -> Vec2 {
        // Scope 1: obtener el Rc del padre (si existe)
        let parent_option = {
            let node = this.borrow();
            node.parent.borrow().upgrade()
        };

        if let Some(parent_rc) = parent_option {
            // Scope 2: pedir el position del padre
            let parent = parent_rc.borrow();
            parent.value.get_position()
        } else {
            Vec2::new(0.0, 0.0)
        }
    }
}

