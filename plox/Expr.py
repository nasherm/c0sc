from typing import *
from .TokenType import Token
class Expr:
	def init(self): pass
	def accept(self,visitor): pass
class ExprVisitor:
	def visitBinaryExpr(self,expr:Expr): pass
	def visitGroupingExpr(self,expr:Expr): pass
	def visitLiteralExpr(self,expr:Expr): pass
	def visitUnaryExpr(self,expr:Expr): pass
	def visitVariableExpr(self,expr:Expr): pass
	def visitAssignExpr(self,expr:Expr): pass

class Binary(Expr):
	def __init__(self,left:Expr, operator:Token, right:Expr):
		self.left = left
		self. operator =  operator
		self. right =  right
	def accept(self, visitor:ExprVisitor):
		return visitor.visitBinaryExpr(self)

class Grouping(Expr):
	def __init__(self,expression:Expr):
		self.expression = expression
	def accept(self, visitor:ExprVisitor):
		return visitor.visitGroupingExpr(self)

class Literal(Expr):
	def __init__(self,value:Any):
		self.value = value
	def accept(self, visitor:ExprVisitor):
		return visitor.visitLiteralExpr(self)

class Unary(Expr):
	def __init__(self,operator:Token, right:Expr):
		self.operator = operator
		self. right =  right
	def accept(self, visitor:ExprVisitor):
		return visitor.visitUnaryExpr(self)

class Variable(Expr):
	def __init__(self,name:Token):
		self.name = name
	def accept(self, visitor:ExprVisitor):
		return visitor.visitVariableExpr(self)

class Assign(Expr):
	def __init__(self,name: Token, value: Expr):
		self.name = name
		self. value =  value
	def accept(self, visitor:ExprVisitor):
		return visitor.visitAssignExpr(self)

