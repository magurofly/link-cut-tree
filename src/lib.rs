use std::rc::*;
use std::cell::*;

pub type RefNode = RefCell<LCTNode>;
pub type RcNode = Rc<RefNode>;
pub type WeakNode = Weak<RefNode>;

pub struct LCTNode {
    parent: Option<WeakNode>,
    children: [Option<RcNode>; 2],
    len: usize,
}

pub trait LinkCutTree: std::ops::Deref<Target = RefNode> {
    fn new() -> RcNode {
        Rc::new(RefCell::new(LCTNode {
            parent: None,
            children: [None, None],
            len: 1,
        }))
    }

    fn ref_rc(&self) -> &RcNode;

    fn get(&self) -> Ref<'_, LCTNode> { self.ref_rc().borrow() }
    fn get_mut(&self) -> RefMut<'_, LCTNode> { self.ref_rc().borrow_mut() }

    fn rc(&self) -> RcNode { Rc::clone(self.ref_rc()) }
    fn weak(&self) -> WeakNode { Rc::downgrade(self.ref_rc()) }

    fn len(&self) -> usize {
        self.get().len
    }

    fn len_mut(&self) -> RefMut<usize> {
        RefMut::map(self.get_mut(), |node| &mut node.len)
    }

    fn parent(&self) -> Option<RcNode> {
        self.get().parent.as_ref().and_then(Weak::upgrade)
    }

    fn parent_mut(&self) -> RefMut<Option<WeakNode>> {
        RefMut::map(self.get_mut(), |node| &mut node.parent)
    }

    fn child(&self, dir: usize) -> Option<RcNode> {
        assert!(dir < 2);
        Some(self.get().children[dir].as_ref()?.rc())
    }

    fn child_mut(&self, dir: usize) -> RefMut<Option<RcNode>> {
        assert!(dir < 2);
        RefMut::map(self.get_mut(), |node| &mut node.children[dir])
    }

    /// 親から見た自分の向き
    fn dir(&self) -> Option<usize> {
        let parent = self.get().parent.as_ref()?.upgrade()?;
        for dir in 0 .. 2 {
            if let Some(child) = &parent.get().children[dir] {
                if Rc::ptr_eq(self.ref_rc(), child) {
                    return Some(dir);
                }
            }
        }
        // 親が light edge でつながっている場合
        None
    }

    fn is_path_root(&self) -> bool {
        self.dir().is_none()
    }

    fn path_parent(&self) -> Option<RcNode> {
        self.dir().and_then(|_| self.parent())
    }

    fn update(&self) {
        let mut len = 1;
        for child in self.get().children.iter() {
            len += child.as_ref().map(|node| node.len()).unwrap_or(0);
        }
        *self.len_mut() = len;
    }

    fn rotate(&self) {
        if let Some(dir) = self.dir() {
            let parent_weak = self.parent_mut().take().unwrap();
            let parent = parent_weak.upgrade().unwrap();
            let child = self.child_mut(1 ^ dir).take();
            if let Some(child) = child.as_ref() {
                *child.parent_mut() = Some(parent_weak.clone());
            }
            *parent.child_mut(dir) = child.clone();
            if let Some(parent_dir) = parent.dir() {
                let ancestor = parent.parent().unwrap();
                *ancestor.child_mut(parent_dir) = Some(self.rc());
            }
            *self.parent_mut() = parent.parent_mut().replace(self.weak());
            parent.update();
            self.update();
        }
    }

    fn splay(&self) {
        while let Some(parent) = self.path_parent() {
            if parent.is_path_root() {
            } else if self.dir() == parent.dir() {
                parent.rotate();
            } else {
                self.rotate();
            }
            self.rotate();
        }
    }

    /// 自身を木の根のパスにつなげ、そのパスの根にする
    fn expose(&self) {
        loop {
            self.splay();
            self.child_mut(1).take();
            self.update();
            if let Some(parent) = self.parent() {
                parent.splay();
                parent.child_mut(1).replace(self.rc());
                parent.update();
            } else {
                break;
            }
        }
    }

    /// 自身の親を new_parent にする
    fn link(&self, new_parent: &Self) {
        self.expose();
        new_parent.expose();
        self.parent_mut().replace(new_parent.weak());
        new_parent.child_mut(1).replace(self.rc());
    }

    /// 自身を親から切り離す
    fn cut(&self) {
        self.child_mut(0).take().unwrap().parent_mut().take();
    }
}

impl LinkCutTree for RcNode {
    fn ref_rc(&self) -> &Self { self }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
